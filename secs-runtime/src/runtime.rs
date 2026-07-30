use alloc::{
    boxed::Box,
    collections::{BTreeMap, VecDeque},
    rc::Rc,
};
use core::cell::RefCell;

use secs_ii::{FunctionId, Secs2Message, StreamId};
use secs_runtime_core::{
    MessageTransport, RuntimeMessage, SecsTimeoutUnit, SecsTimer, SystemByteSource, TaskQueue,
    TaskRunner,
};

use crate::{
    error::{CallError, HandlerError, SecsRuntimeError},
    scenario::{ScenarioContext, SecsScenario, handler::BoxedSecsScenario},
    service::{SecsService, ServiceContext},
    shared::{RuntimeCommand, RuntimeHandle, RuntimeShared},
    timer::TimeoutConfig,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SecsRuntimeRoute {
    pub stream: StreamId,
    pub function: FunctionId,
}

impl SecsRuntimeRoute {
    pub fn new(stream: StreamId, function: FunctionId) -> Self {
        Self { stream, function }
    }

    pub fn from_message(message: &Secs2Message) -> Self {
        Self::new(message.stream, message.function)
    }
}

pub type ScenarioTaskOutput = Result<(), HandlerError>;
pub type DefaultScenarioTaskRunner = TaskQueue<ScenarioTaskOutput>;

pub struct SecsRuntime<R, S = DefaultScenarioTaskRunner>
where
    R: SecsTimer,
    S: TaskRunner<ScenarioTaskOutput>,
{
    // Owns the SECS transport state machine and completed message queues.
    transport: Box<dyn MessageTransport>,
    // Timer backend is injected so std/no_std runtimes can provide their own clock.
    timer: R,
    timeout_config: TimeoutConfig<R::Duration>,
    // Scenario/service contexts share this state to enqueue sends and register recv waiters.
    shared: Rc<RefCell<RuntimeShared>>,
    // Primary messages are routed to services by SxFy.
    services: BTreeMap<SecsRuntimeRoute, Box<dyn SecsService>>,
    // Scenario futures are resumed by promise wakers instead of being polled every tick.
    task_runner: S,
    // Messages that are not matched by pending calls or services are exposed to callers.
    incomming_msgs: VecDeque<Secs2Message>,
}

impl<R> SecsRuntime<R, DefaultScenarioTaskRunner>
where
    R: SecsTimer,
{
    pub fn new(
        transport: impl MessageTransport + 'static,
        timer: R,
        system_bytes: SystemByteSource,
        timeout_config: TimeoutConfig<R::Duration>,
    ) -> Self {
        Self::with_task_runner(
            transport,
            timer,
            system_bytes,
            timeout_config,
            TaskQueue::new(),
        )
    }
}

impl<R, S> SecsRuntime<R, S>
where
    R: SecsTimer,
    S: TaskRunner<ScenarioTaskOutput>,
{
    pub fn with_task_runner(
        transport: impl MessageTransport + 'static,
        timer: R,
        system_bytes: SystemByteSource,
        timeout_config: TimeoutConfig<R::Duration>,
        task_runner: S,
    ) -> Self {
        Self::with_boxed_parts(
            Box::new(transport),
            timer,
            system_bytes,
            timeout_config,
            task_runner,
        )
    }

    pub fn with_boxed_parts(
        transport: Box<dyn MessageTransport>,
        timer: R,
        system_bytes: SystemByteSource,
        timeout_config: TimeoutConfig<R::Duration>,
        task_runner: S,
    ) -> Self {
        Self {
            transport,
            timer,
            timeout_config,
            shared: Rc::new(RefCell::new(RuntimeShared::new(system_bytes))),
            services: BTreeMap::new(),
            task_runner,
            incomming_msgs: VecDeque::new(),
        }
    }

    fn handle(&self) -> RuntimeHandle {
        RuntimeHandle::new(self.shared.clone())
    }

    pub fn register_service<H>(&mut self, stream: StreamId, function: FunctionId, service: H)
    where
        H: SecsService + 'static,
    {
        self.services
            .insert(SecsRuntimeRoute::new(stream, function), Box::new(service));
    }

    pub fn start_scenario<H>(&mut self, scenario: H) -> Result<(), SecsRuntimeError<R::Error>>
    where
        H: SecsScenario + 'static,
    {
        self.start_boxed_scenario(Box::new(scenario))
    }

    fn start_boxed_scenario(
        &mut self,
        scenario: Box<dyn BoxedSecsScenario>,
    ) -> Result<(), SecsRuntimeError<R::Error>> {
        let ctx = ScenarioContext::new(self.handle());
        self.task_runner
            .spawn_boxed(scenario.run_boxed(ctx))
            .map_err(SecsRuntimeError::TaskSpawn)
    }

    pub fn poll_received(&mut self) -> Option<Secs2Message> {
        self.incomming_msgs.pop_front()
    }

    pub fn start(&mut self) -> Result<(), SecsRuntimeError<R::Error>> {
        self.transport.start().map_err(SecsRuntimeError::Transport)
    }

    pub fn tick(&mut self) -> Result<(), SecsRuntimeError<R::Error>>
    where
        R::Duration: Copy,
    {
        // Drain transport/timer events first, then run ready scenario tasks. Commands produced by
        // those tasks are flushed once more before the tick returns.
        self.transport.poll().map_err(SecsRuntimeError::Transport)?;
        self.start_transport_timeouts()?;
        self.handle_expired_timeouts()?;
        self.complete_expired_timeout_calls();
        self.process_recv_messages();
        self.process_commands()?;
        self.poll_tasks()?;
        self.process_commands()
    }

    fn start_transport_timeouts(&mut self) -> Result<(), SecsRuntimeError<R::Error>>
    where
        R::Duration: Copy,
    {
        while let Some(ticket) = self.transport.poll_timeout() {
            let duration = self.timeout_config.duration_for(ticket.timeout);
            self.timer
                .start_timeout(ticket, duration)
                .map_err(SecsRuntimeError::Timer)?;
        }
        Ok(())
    }

    fn handle_expired_timeouts(&mut self) -> Result<(), SecsRuntimeError<R::Error>> {
        while let Some(ticket) = self.timer.poll_timeout().map_err(SecsRuntimeError::Timer)? {
            if let Err(error) = self.transport.handle_timeout(ticket) {
                self.complete_expired_timeout_calls();
                return Err(SecsRuntimeError::Transport(error));
            }
            self.complete_expired_timeout_calls();
        }
        Ok(())
    }

    fn process_recv_messages(&mut self) {
        while let Some(message) = self.transport.poll_recv() {
            self.process_recv_message(message);
        }
    }

    fn process_recv_message(&mut self, message: RuntimeMessage) {
        let key = message.transaction_key;

        // Pending calls consume matching secondary replies. If no recv waiter exists, the caller
        // intentionally ignored the response, so the pending slot is simply cleared.
        if let Some(pending) = self.shared.borrow_mut().pending_calls.remove(&key) {
            if let Some(resolver) = pending.resolver {
                resolver.resolve(message.into_payload());
            }
            return;
        }

        if message.is_primary() {
            self.serve_message(message);
        } else {
            self.incomming_msgs.push_back(message.into_payload());
        }
    }

    fn complete_expired_timeout_calls(&mut self) {
        while let Some(timeout) = self.transport.poll_expired_timeout() {
            log::error!("timeout occured! {:?}", timeout);

            match timeout {
                SecsTimeoutUnit::T3(key) => {
                    if let Some(pending) = self.shared.borrow_mut().pending_calls.remove(&key)
                        && let Some(resolver) = pending.resolver
                    {
                        resolver.reject(CallError::Timeout);
                    }
                }
                SecsTimeoutUnit::T6 | SecsTimeoutUnit::T7 | SecsTimeoutUnit::T8 => {
                    let mut shared = self.shared.borrow_mut();
                    let pending_calls = core::mem::take(&mut shared.pending_calls);
                    for (_, pending) in pending_calls {
                        if let Some(resolver) = pending.resolver {
                            resolver.reject(CallError::Timeout);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn serve_message(&mut self, message: RuntimeMessage) {
        let handle = self.handle();
        let route = SecsRuntimeRoute::from_message(&message.payload);
        let Some(service) = self.services.get_mut(&route) else {
            let secs2_msg = message.into_payload();
            log::warn!(
                "no handler found for msg: S{}F{}",
                secs2_msg.stream.0,
                secs2_msg.function.0
            );
            self.incomming_msgs.push_back(secs2_msg);
            return;
        };

        let mut ctx = ServiceContext::new(handle, message);
        if let Err(error) = service.serve(&mut ctx) {
            log::error!("service failed: {:?}", error);
        }
    }

    fn process_commands(&mut self) -> Result<(), SecsRuntimeError<R::Error>> {
        loop {
            let command = { self.shared.borrow_mut().commands.pop_front() };
            let Some(command) = command else {
                return Ok(());
            };

            match command {
                RuntimeCommand::Send { message, call } => {
                    let result = self.transport.send(message);
                    if let Err(error) = result {
                        // A send failure completes the waiting promise immediately instead of
                        // letting the transaction wait for a later timeout.
                        if let Some(key) = call {
                            if let Some(pending) =
                                self.shared.borrow_mut().pending_calls.remove(&key)
                                && let Some(resolver) = pending.resolver
                            {
                                resolver.reject(CallError::Transport(error));
                            }
                        }
                        return Err(SecsRuntimeError::Transport(error));
                    }
                }
            }
        }
    }

    fn poll_tasks(&mut self) -> Result<(), SecsRuntimeError<R::Error>> {
        // Only tasks woken by promise resolvers are polled here.
        for result in self.task_runner.poll_completed() {
            match result {
                Ok(()) => {}
                Err(error) => return Err(SecsRuntimeError::Handler(error)),
            }
        }
        Ok(())
    }
}

use alloc::{
    boxed::Box,
    collections::{BTreeMap, VecDeque},
    rc::Rc,
};
use core::{
    cell::RefCell,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use secs_ii::{FunctionId, Secs2Message, StreamId};
use secs_runtime_core::{MessageTransport, RuntimeMessage, SecsTimer, SystemByteSource};

use crate::{
    error::{CallError, HandlerError, SecsRuntimeError},
    scenario::{BoxFuture, ScenarioContext, SecsScenario},
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

struct RuntimeTask {
    future: BoxFuture<'static, Result<(), HandlerError>>,
}

pub struct SecsRuntime<T, R>
where
    R: SecsTimer,
{
    transport: T,
    timer: R,
    timeout_config: TimeoutConfig<R::Duration>,
    shared: Rc<RefCell<RuntimeShared>>,
    services: BTreeMap<SecsRuntimeRoute, Box<dyn SecsService>>,
    tasks: VecDeque<RuntimeTask>,
    // 들어온 메시지
    incomming_msgs: VecDeque<Secs2Message>,
}

impl<T, R> SecsRuntime<T, R>
where
    R: SecsTimer,
{
    pub fn new(
        transport: T,
        timer: R,
        system_bytes: SystemByteSource,
        timeout_config: TimeoutConfig<R::Duration>,
    ) -> Self {
        Self {
            transport,
            timer,
            timeout_config,
            shared: Rc::new(RefCell::new(RuntimeShared::new(system_bytes))),
            services: BTreeMap::new(),
            tasks: VecDeque::new(),
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

    pub fn spawn_scenario<H>(&mut self, scenario: H)
    where
        H: SecsScenario + 'static,
    {
        let ctx = ScenarioContext::new(self.handle());
        self.tasks.push_back(RuntimeTask {
            future: Box::new(scenario).run(ctx),
        });
    }

    pub fn poll_incomming_msg(&mut self) -> Option<Secs2Message> {
        self.incomming_msgs.pop_front()
    }
}

impl<T, R> SecsRuntime<T, R>
where
    T: MessageTransport,
    R: SecsTimer,
    R::Duration: Copy,
{
    pub fn start(&mut self) -> Result<(), SecsRuntimeError<R::Error>> {
        self.transport.start().map_err(SecsRuntimeError::Transport)
    }

    pub fn tick(&mut self) -> Result<(), SecsRuntimeError<R::Error>> {
        self.transport.poll().map_err(SecsRuntimeError::Transport)?;
        self.start_transport_timeouts()?;
        self.handle_expired_timeouts()?;
        self.process_recv_messages()?;
        self.process_commands()?;
        self.poll_tasks()?;
        self.process_commands()
    }

    fn start_transport_timeouts(&mut self) -> Result<(), SecsRuntimeError<R::Error>> {
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
            self.transport
                .handle_timeout(ticket)
                .map_err(SecsRuntimeError::Transport)?;
        }
        Ok(())
    }

    fn process_recv_messages(&mut self) -> Result<(), SecsRuntimeError<R::Error>> {
        while let Some(message) = self.transport.poll_recv() {
            let Some(message) = self.complete_pending_call(message)? else {
                continue;
            };

            if message.is_primary() {
                self.serve_message(message);
            } else {
                self.incomming_msgs.push_back(message.into_payload());
            }
        }
        Ok(())
    }

    fn complete_pending_call(
        &mut self,
        message: RuntimeMessage,
    ) -> Result<Option<RuntimeMessage>, SecsRuntimeError<R::Error>> {
        let key = message.transaction_key;
        if !self.shared.borrow().pending_calls.contains_key(&key) {
            return Ok(Some(message));
        }

        let mut shared = self.shared.borrow_mut();
        if let Some(pending) = shared.pending_calls.get_mut(&key) {
            pending.result = Some(Ok(message.into_payload()));
        }
        Ok(None)
    }

    /// 내부에 등록된 서비스 호출
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
                        if let Some(key) = call {
                            if let Some(pending) =
                                self.shared.borrow_mut().pending_calls.get_mut(&key)
                            {
                                pending.result = Some(Err(CallError::Transport(error)));
                            }
                        }
                        return Err(SecsRuntimeError::Transport(error));
                    }
                }
            }
        }
    }

    fn poll_tasks(&mut self) -> Result<(), SecsRuntimeError<R::Error>> {
        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);
        let mut remaining = VecDeque::new();

        while let Some(mut task) = self.tasks.pop_front() {
            match task.future.as_mut().poll(&mut cx) {
                Poll::Ready(Ok(())) => {}
                Poll::Ready(Err(error)) => return Err(SecsRuntimeError::Handler(error)),
                Poll::Pending => remaining.push_back(task),
            }
        }

        self.tasks = remaining;
        Ok(())
    }
}

fn noop_waker() -> Waker {
    unsafe fn clone(_: *const ()) -> RawWaker {
        RawWaker::new(core::ptr::null(), &VTABLE)
    }
    unsafe fn wake(_: *const ()) {}
    unsafe fn wake_by_ref(_: *const ()) {}
    unsafe fn drop(_: *const ()) {}

    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

    unsafe { Waker::from_raw(RawWaker::new(core::ptr::null(), &VTABLE)) }
}

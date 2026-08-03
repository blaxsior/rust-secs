use alloc::boxed::Box;
use std::sync::{Arc, Mutex, MutexGuard};

use secs_runtime_core::{
    MessageTransport, RuntimeMessage, SecsTimeoutUnit, SecsTimer, SystemByteSource,
};

use crate::{
    error::{CallError, SecsRuntimeError},
    shared::{RuntimeCommand, RuntimeShared, SecsHandle},
    timer::TimeoutConfig,
};

pub struct SecsRuntime<R, T = Box<dyn MessageTransport>>
where
    R: SecsTimer,
    T: MessageTransport,
{
    // Owns the SECS transport state machine and completed message queues.
    transport: T,
    // Timer backend is injected so std/no_std runtimes can provide their own clock.
    timer: R,
    timeout_config: TimeoutConfig<R::Duration>,
    // Handles share this state to enqueue sends and register recv waiters.
    shared: Arc<Mutex<RuntimeShared>>,
}

impl<R, T> SecsRuntime<R, T>
where
    R: SecsTimer,
    T: MessageTransport,
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
            shared: Arc::new(Mutex::new(RuntimeShared::new(system_bytes))),
        }
    }

    pub fn handle(&self) -> SecsHandle {
        SecsHandle::new(self.shared.clone())
    }

    pub fn start(&mut self) -> Result<(), SecsRuntimeError<R::Error>> {
        self.transport.start().map_err(SecsRuntimeError::Transport)
    }

    pub fn tick(&mut self) -> Result<(), SecsRuntimeError<R::Error>>
    where
        R::Duration: Copy,
    {
        self.transport.poll().map_err(SecsRuntimeError::Transport)?;
        self.start_transport_timeouts()?;
        self.handle_expired_timeouts()?;
        self.complete_expired_timeout_calls();
        self.process_recv_messages();
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
        let mut shared = self.shared();
        if let Some(pending) = shared.pending_calls.remove(&key) {
            if let Some(resolver) = pending.resolver {
                resolver.resolve(message.payload);
            }
        } else {
            shared.push_incomming(message);
        }
    }

    fn complete_expired_timeout_calls(&mut self) {
        while let Some(timeout) = self.transport.poll_expired_timeout() {
            log::error!("timeout occured! {:?}", timeout);

            match timeout {
                SecsTimeoutUnit::T3(key) => {
                    if let Some(pending) = self.shared().pending_calls.remove(&key)
                        && let Some(resolver) = pending.resolver
                    {
                        resolver.reject(CallError::Timeout);
                    }
                }
                SecsTimeoutUnit::T6 | SecsTimeoutUnit::T7 | SecsTimeoutUnit::T8 => {
                    let mut shared = self.shared();
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

    fn process_commands(&mut self) -> Result<(), SecsRuntimeError<R::Error>> {
        loop {
            let command = { self.shared().commands.pop_front() };
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
                                self.shared().pending_calls.remove(&key)
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

    fn shared(&self) -> MutexGuard<'_, RuntimeShared> {
        self.shared
            .lock()
            .expect("runtime shared state mutex poisoned")
    }
}

impl<R> SecsRuntime<R, Box<dyn MessageTransport>>
where
    R: SecsTimer,
{
    pub fn with_boxed_transport(
        transport: Box<dyn MessageTransport>,
        timer: R,
        system_bytes: SystemByteSource,
        timeout_config: TimeoutConfig<R::Duration>,
    ) -> Self {
        Self::new(transport, timer, system_bytes, timeout_config)
    }
}

use secs_ii::Secs2Message;

use crate::core::{ByteDataSource, CorrelationIdSource, RuntimeTimer};
use crate::message::{MessageMachine, MessageRuntime, MessageRuntimeTick, RuntimeError};
use crate::secs2::{
    CallToken, Secs2MessageHandler, Secs2MessageLayer, Secs2Runtime, Secs2RuntimeError,
    Secs2RuntimeEvent,
};

pub enum SecsMachineError<D, M, T, H> {
    Runtime(RuntimeError<D, M, T>),
    Secs2(Secs2RuntimeError<M, H>),
}

pub struct SecsMachine<D, M, T, C, H> {
    runtime: Secs2Runtime<MessageRuntime<D, M, T>, C>,
    handler: H,
}

impl<D, M, T, C, H> SecsMachine<D, M, T, C, H> {
    pub fn new(datasource: D, machine: M, timer: T, correlation_ids: C, handler: H) -> Self {
        let lower = MessageRuntime::new(datasource, machine, timer);
        let runtime = Secs2Runtime::new(lower, correlation_ids);

        Self { runtime, handler }
    }

    pub fn runtime(&self) -> &Secs2Runtime<MessageRuntime<D, M, T>, C> {
        &self.runtime
    }

    pub fn runtime_mut(&mut self) -> &mut Secs2Runtime<MessageRuntime<D, M, T>, C> {
        &mut self.runtime
    }

    pub fn handler(&self) -> &H {
        &self.handler
    }

    pub fn handler_mut(&mut self) -> &mut H {
        &mut self.handler
    }

    pub fn into_parts(self) -> (Secs2Runtime<MessageRuntime<D, M, T>, C>, H) {
        (self.runtime, self.handler)
    }
}

impl<D, M, T, C, H> SecsMachine<D, M, T, C, H>
where
    D: ByteDataSource,
    M: MessageMachine,
    T: RuntimeTimer<Timeout = M::Timeout, Ticket = M::TimeoutTicket>,
{
    pub fn start(
        &mut self,
    ) -> Result<(), SecsMachineError<D::Error, M::Error, T::Error, H::Error>>
    where
        H: Secs2MessageHandler,
    {
        self.runtime
            .lower_mut()
            .arm_machine_timeouts()
            .map_err(RuntimeError::Timer)
            .map_err(SecsMachineError::Runtime)
    }

    pub fn tick(
        &mut self,
        read_buf: &mut [u8],
    ) -> Result<MessageRuntimeTick, SecsMachineError<D::Error, M::Error, T::Error, H::Error>>
    where
        H: Secs2MessageHandler,
    {
        self.runtime
            .lower_mut()
            .tick(read_buf)
            .map_err(SecsMachineError::Runtime)
    }
}

impl<D, M, T, C, H> SecsMachine<D, M, T, C, H>
where
    D: ByteDataSource,
    M: MessageMachine,
    T: RuntimeTimer,
    C: CorrelationIdSource,
    H: Secs2MessageHandler,
    MessageRuntime<D, M, T>: Secs2MessageLayer<Error = M::Error, Event = M::Event>,
{
    pub fn start_call(
        &mut self,
        primary: Secs2Message,
    ) -> Result<CallToken, SecsMachineError<D::Error, M::Error, T::Error, H::Error>> {
        self.runtime
            .start_call(primary)
            .map_err(Secs2RuntimeError::Lower)
            .map_err(SecsMachineError::Secs2)
    }

    pub fn poll_call(
        &mut self,
        token: CallToken,
    ) -> Result<Option<Secs2Message>, SecsMachineError<D::Error, M::Error, T::Error, H::Error>> {
        self.runtime
            .poll_call(token, &mut self.handler)
            .map_err(SecsMachineError::Secs2)
    }

    pub fn call(
        &mut self,
        primary: Secs2Message,
    ) -> Result<Option<Secs2Message>, SecsMachineError<D::Error, M::Error, T::Error, H::Error>> {
        self.runtime
            .call(primary, &mut self.handler)
            .map_err(SecsMachineError::Secs2)
    }

    pub fn handle(
        &mut self,
    ) -> Result<Option<Secs2RuntimeEvent<M::Event>>, SecsMachineError<D::Error, M::Error, T::Error, H::Error>>
    {
        self.runtime
            .handle(&mut self.handler)
            .map_err(SecsMachineError::Secs2)
    }

    pub fn poll_event(
        &mut self,
    ) -> Result<Option<Secs2RuntimeEvent<M::Event>>, SecsMachineError<D::Error, M::Error, T::Error, H::Error>>
    {
        self.runtime
            .poll_event(&mut self.handler)
            .map_err(SecsMachineError::Secs2)
    }
}

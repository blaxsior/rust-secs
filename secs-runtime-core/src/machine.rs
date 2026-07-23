use crate::message::RuntimeMessage;
use crate::timer::TimeoutTicket;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachineSignal {
    LinkOpened,
    LinkClosed,
    WriteCompleted,
    WriteFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachineEvent {
    LinkOpenRequested,
    LinkCloseRequested,
}

/// 메시지 전송을 담당하는 구조체. 통신 방식은 내부에 숨기고 있음
pub trait MessageTransport {
    /// Message
    fn start(&mut self);

    fn handle_write_message(
        &mut self,
        msg: RuntimeMessage,
    ) -> Result<(), crate::error::MachineError>;

    fn handle_signal(&mut self, signal: MachineSignal) -> Result<(), crate::error::MachineError>;

    fn handle_timeout(&mut self, ticket: TimeoutTicket) -> Result<(), crate::error::MachineError>;

    fn tick(&mut self) -> Result<(), crate::error::MachineError> {
        Ok(())
    }

    fn poll_read_message(&mut self) -> Option<RuntimeMessage>;

    fn poll_event(&mut self) -> Option<MachineEvent>;

    fn poll_timeout(&mut self) -> Option<TimeoutTicket>;
}

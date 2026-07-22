use alloc::vec::Vec;

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

pub trait MessageMachine {
    fn handle_read_bytes(&mut self, bytes: &[u8]) -> Result<(), crate::error::MachineError>;

    fn handle_write_message(
        &mut self,
        msg: RuntimeMessage,
    ) -> Result<(), crate::error::MachineError>;

    fn handle_signal(&mut self, signal: MachineSignal) -> Result<(), crate::error::MachineError>;

    fn handle_timeout(&mut self, ticket: TimeoutTicket) -> Result<(), crate::error::MachineError>;

    fn poll_read_message(&mut self) -> Option<RuntimeMessage>;

    fn poll_write_bytes(&mut self) -> Option<Vec<u8>>;

    fn poll_event(&mut self) -> Option<MachineEvent>;

    fn poll_timeout(&mut self) -> Option<TimeoutTicket>;
}

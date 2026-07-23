use alloc::vec::Vec;
use secs_common::{ConnectionRole, TimeoutTicket};
use secs_runtime_core::{MachineEvent, MessageTransport};

use crate::transport::hsms::{HsmsMessage, protocol::{connection::{HsmsConnectionState, HsmsSessionManager}, message::{HsmsMessageMachine, HsmsWrite}}};

pub mod assembler;
pub mod connection;
pub mod message;

pub struct HsmsTransport {
    state: HsmsConnectionState,
    role: ConnectionRole,
    message: HsmsMessageMachine,
    // outgoing_writes: Vec<HsmsWrite>,
    outgoing_messages: Vec<HsmsMessage>,
    outgoing_events: Vec<MachineEvent>,
    outgoing_timeouts: Vec<TimeoutTicket>
}

impl MessageTransport for HsmsTransport {
    fn start(&mut self) {
        todo!()
    }

    fn handle_write_message(
        &mut self,
        msg: secs_runtime_core::RuntimeMessage,
    ) -> Result<(), secs_runtime_core::error::MachineError> {
        todo!()
    }

    fn handle_signal(&mut self, signal: secs_runtime_core::MachineSignal) -> Result<(), secs_runtime_core::error::MachineError> {
        todo!()
    }

    fn handle_timeout(&mut self, ticket: TimeoutTicket) -> Result<(), secs_runtime_core::error::MachineError> {
        todo!()
    }

    fn poll_read_message(&mut self) -> Option<secs_runtime_core::RuntimeMessage> {
        todo!()
    }

    fn poll_event(&mut self) -> Option<MachineEvent> {
        todo!()
    }

    fn poll_timeout(&mut self) -> Option<TimeoutTicket> {
        todo!()
    }
}

impl HsmsSessionManager for HsmsTransport {

}
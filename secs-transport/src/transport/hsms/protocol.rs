use alloc::collections::VecDeque;
use secs_common::{TimeoutTicket};
use secs_runtime_core::{MachineEvent, MessageTransport};

use crate::transport::hsms::{HsmsMessage, config::HsmsTransportConfig, protocol::message::HsmsMessageMachine};

pub mod assembler;
pub mod connection;
pub mod message;

pub struct HsmsTransport {

    message: HsmsMessageMachine,
    // outgoing_writes: Vec<HsmsWrite>,
    outgoing_messages: VecDeque<HsmsMessage>,
    outgoing_events: VecDeque<MachineEvent>,
    outgoing_timeouts: VecDeque<TimeoutTicket>,
}

impl HsmsTransport {
    pub fn new(config: &HsmsTransportConfig) -> Self {
        Self {
            message: HsmsMessageMachine::new(config),
            outgoing_messages: VecDeque::new(),
            outgoing_events: VecDeque::new(),
            outgoing_timeouts: VecDeque::new()
        }
    }
}

impl MessageTransport for HsmsTransport {
    fn start(&mut self) {
        todo!()
    }
    
    fn write(
        &mut self,
        msg: secs_runtime_core::RuntimeMessage,
    ) -> Result<(), secs_runtime_core::error::MachineError> {
        todo!()
    }
    
    fn read(&mut self) -> Option<secs_runtime_core::RuntimeMessage> {
        todo!()
    }

    fn handle_timeout(&mut self, ticket: TimeoutTicket) -> Result<(), secs_runtime_core::error::MachineError> {
        todo!()
    }
    
    fn poll_timeout(&mut self) -> Option<TimeoutTicket> {
        todo!()
    }

   
}
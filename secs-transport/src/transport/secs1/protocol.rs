use crate::transport::secs1::{
    config::Secs1TransportConfig,
    protocol::{block_transfer::Secs1BlockTransferMachine, message::Secs1MessageMachine},
};
use alloc::vec::Vec;
use secs_runtime_core::MessageMachine;

pub mod block_transfer;
pub mod message;

pub struct Secs1Machine {
    block_transfer: Secs1BlockTransferMachine,
    message: Secs1MessageMachine,
    config: Secs1TransportConfig,
}

impl Secs1Machine {
    pub fn new(config: Secs1TransportConfig) -> Self {
        let block_transfer = Secs1BlockTransferMachine::new(&config);
        let message = Secs1MessageMachine::new(&config);

        Self {
            block_transfer,
            message,
            config,
        }
    }
}

impl MessageMachine for Secs1Machine {
    fn handle_read_bytes(
        &mut self,
        bytes: &[u8],
    ) -> Result<(), secs_runtime_core::error::MachineError> {
        todo!()
    }

    fn handle_write_message(
        &mut self,
        msg: secs_runtime_core::RuntimeMessage,
    ) -> Result<(), secs_runtime_core::error::MachineError> {
        todo!()
    }

    fn handle_signal(
        &mut self,
        signal: secs_runtime_core::MachineSignal,
    ) -> Result<(), secs_runtime_core::error::MachineError> {
        todo!()
    }

    fn handle_timeout(
        &mut self,
        ticket: secs_runtime_core::TimeoutTicket,
    ) -> Result<(), secs_runtime_core::error::MachineError> {
        todo!()
    }

    fn poll_read_message(&mut self) -> Option<secs_runtime_core::RuntimeMessage> {
        todo!()
    }

    fn poll_write_bytes(&mut self) -> Option<Vec<u8>> {
        todo!()
    }

    fn poll_event(&mut self) -> Option<secs_runtime_core::MachineEvent> {
        todo!()
    }

    fn poll_timeout(&mut self) -> Option<secs_runtime_core::TimeoutTicket> {
        todo!()
    }
}

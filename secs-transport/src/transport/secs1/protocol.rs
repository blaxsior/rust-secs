use alloc::vec::Vec;
use sansio::Protocol;

use crate::transport::{
    SecsTimeoutUnit, TimeoutTicket,
    error::SecsTransportError,
    secs1::{
        Secs1Message,
        config::Secs1TransportConfig,
        protocol::{
            block_transfer::{Secs1BlockTransferEvent, Secs1BlockTransferMachine},
            message::{Secs1MessageEvent, Secs1MessageMachine, Secs1MessageSignal},
        },
    },
};

pub mod block_transfer;
pub mod message;

/// SECS-I byte stream and message protocol composer.
///
/// This machine stays at the transport-message level:
/// bytes <-> blocks <-> Secs1Message.
pub struct Secs1Machine {
    block_transfer: Secs1BlockTransferMachine,
    message: Secs1MessageMachine,
}

impl Secs1Machine {
    pub fn new(config: Secs1TransportConfig) -> Self {
        let block_transfer = Secs1BlockTransferMachine::new(&config);
        let message = Secs1MessageMachine::new(&config);

        Self {
            block_transfer,
            message,
        }
    }

    fn handle_read_bytes(&mut self, bytes: &[u8]) -> Result<(), SecsTransportError> {
        self.block_transfer.handle_read(bytes)?;
        self.pump()
    }

    fn handle_write_message(&mut self, msg: Secs1Message) -> Result<(), SecsTransportError> {
        self.message.handle_write(msg)?;
        self.pump()
    }

    fn handle_timeout(&mut self, ticket: TimeoutTicket) -> Result<(), SecsTransportError> {
        match ticket.timeout {
            SecsTimeoutUnit::T1 | SecsTimeoutUnit::T2 => {
                self.block_transfer.handle_timeout(ticket)?;
            }
            SecsTimeoutUnit::T3(_) | SecsTimeoutUnit::T4(_) => {
                self.message.handle_timeout(ticket)?;
            }
            _ => {}
        }

        self.pump()
    }

    fn poll_read_message(&mut self) -> Option<Secs1Message> {
        let _ = self.pump();
        self.message.poll_read()
    }

    fn poll_write_bytes(&mut self) -> Option<Vec<u8>> {
        let _ = self.pump();
        self.block_transfer.poll_write()
    }

    fn poll_timeout(&mut self) -> Option<TimeoutTicket> {
        self.block_transfer
            .poll_timeout()
            .or_else(|| self.message.poll_timeout())
    }

    fn pump(&mut self) -> Result<(), SecsTransportError> {
        loop {
            let mut progressed = false;

            while let Some(block) = self.block_transfer.poll_read() {
                self.message.handle_read(block)?;
                progressed = true;
            }

            while let Some(event) = self.block_transfer.poll_event() {
                self.handle_block_transfer_event(event)?;
                progressed = true;
            }

            while let Some(block) = self.message.poll_write() {
                self.block_transfer.handle_write(block)?;
                progressed = true;
            }

            if !progressed {
                break;
            }
        }

        Ok(())
    }

    fn handle_block_transfer_event(
        &mut self,
        event: Secs1BlockTransferEvent,
    ) -> Result<(), SecsTransportError> {
        match event {
            Secs1BlockTransferEvent::SendSuccess { header } => {
                self.message
                    .handle_event(Secs1MessageSignal::BlockSendSuccess { header })?;
                Ok(())
            }
            Secs1BlockTransferEvent::SendFailed { header, .. } => {
                self.message
                    .handle_event(Secs1MessageSignal::BlockSendFailed { header })?;
                Ok(())
            }
            Secs1BlockTransferEvent::ReceiveFailed { error }
            | Secs1BlockTransferEvent::ErrorOccurred(error) => Err(error),
            Secs1BlockTransferEvent::Retry(_) => Ok(()),
        }
    }
}

impl Protocol<&[u8], Secs1Message, ()> for Secs1Machine {
    type Rout = Secs1Message;
    type Wout = Vec<u8>;
    type Eout = Secs1MessageEvent;
    type Error = SecsTransportError;
    type Time = TimeoutTicket;

    fn handle_read(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        self.handle_read_bytes(bytes)
    }

    fn poll_read(&mut self) -> Option<Self::Rout> {
        self.poll_read_message()
    }

    fn handle_write(&mut self, msg: Secs1Message) -> Result<(), Self::Error> {
        self.handle_write_message(msg)
    }

    fn poll_write(&mut self) -> Option<Self::Wout> {
        self.poll_write_bytes()
    }

    fn handle_timeout(&mut self, timeticket: Self::Time) -> Result<(), Self::Error> {
        Secs1Machine::handle_timeout(self, timeticket)
    }

    fn poll_timeout(&mut self) -> Option<Self::Time> {
        Secs1Machine::poll_timeout(self)
    }

    fn handle_event(&mut self, _event: ()) -> Result<(), Self::Error> {
        Ok(())
    }

    fn poll_event(&mut self) -> Option<Self::Eout> {
        self.message.poll_event()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use core::time::Duration;
    use secs_ii::{FunctionId, StreamId, item::Secs2Variant};

    use crate::transport::{
        ConnectionRole, DeviceId, Rbit, SecsTimeoutUnit, SystemByte, Wbit,
        secs1::{Secs1Message, Secs1MessageHeader, block::Secs1HandshakeCode, convert::encode},
    };

    fn build_config() -> Secs1TransportConfig {
        Secs1TransportConfig {
            device_id: DeviceId(10),
            local_role: ConnectionRole::Active,
            t1_timeout: Duration::from_millis(100),
            t2_timeout: Duration::from_millis(100),
            t3_timeout: Duration::from_millis(100),
            t4_timeout: Duration::from_millis(100),
            t2_rty_limit: 3,
        }
    }

    fn build_body() -> Secs2Variant {
        Secs2Variant::list(vec![Secs2Variant::uint4(3001), Secs2Variant::uint4(3002)])
    }

    fn build_local_primary(config: &Secs1TransportConfig, system_byte: SystemByte) -> Secs1Message {
        Secs1Message::new(
            Secs1MessageHeader {
                device_id: config.device_id,
                rbit: Rbit::FORWARD,
                wbit: Wbit(false),
                stream: StreamId(1),
                function: FunctionId(3),
                system_byte,
            },
            Some(build_body()),
        )
    }

    fn build_remote_primary_bytes(
        config: &Secs1TransportConfig,
        system_byte: SystemByte,
    ) -> Vec<u8> {
        let msg = Secs1Message::new(
            Secs1MessageHeader {
                device_id: config.device_id,
                rbit: Rbit::REVERSE,
                wbit: Wbit(false),
                stream: StreamId(1),
                function: FunctionId(3),
                system_byte,
            },
            Some(build_body()),
        );

        let block = encode(msg).unwrap().into_iter().next().unwrap();
        block.to_bytes()
    }

    #[test]
    fn test_secs1_message_to_bytes() {
        let config = build_config();
        let system_byte = SystemByte(101);
        let msg = build_local_primary(&config, system_byte);
        let mut machine = Secs1Machine::new(config);

        machine.handle_write_message(msg).unwrap();

        let enq = machine.poll_write_bytes().unwrap();
        assert_eq!(enq, vec![Secs1HandshakeCode::ENQ.into()]);

        let timeout = machine.poll_timeout().unwrap();
        assert_eq!(timeout.timeout, SecsTimeoutUnit::T2);

        machine
            .handle_read_bytes(&[Secs1HandshakeCode::EOT.into()])
            .unwrap();

        let block_bytes = machine.poll_write_bytes().unwrap();
        assert!(block_bytes.len() > 10);
        assert_eq!(block_bytes[0] as usize, block_bytes.len() - 3);

        machine
            .handle_read_bytes(&[Secs1HandshakeCode::ACK.into()])
            .unwrap();

        assert!(machine.poll_read_message().is_none());
    }

    #[test]
    fn test_bytes_to_secs1_message() {
        let config = build_config();
        let system_byte = SystemByte(202);
        let block_bytes = build_remote_primary_bytes(&config, system_byte);
        let mut machine = Secs1Machine::new(config);

        machine
            .handle_read_bytes(&[Secs1HandshakeCode::ENQ.into()])
            .unwrap();

        let eot = machine.poll_write_bytes().unwrap();
        assert_eq!(eot, vec![Secs1HandshakeCode::EOT.into()]);

        machine.handle_read_bytes(&block_bytes).unwrap();

        let ack = machine.poll_write_bytes().unwrap();
        assert_eq!(ack, vec![Secs1HandshakeCode::ACK.into()]);

        let msg = machine.poll_read_message().unwrap();
        assert_eq!(msg.header.system_byte, system_byte);
        assert_eq!(msg.header.stream, StreamId(1));
        assert_eq!(msg.header.function, FunctionId(3));
        assert!(!msg.header.wbit.need_reply());
        assert!(msg.body.is_some());
    }
}

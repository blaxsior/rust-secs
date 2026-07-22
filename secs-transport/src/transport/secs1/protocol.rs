use alloc::vec::Vec;
use sansio::Protocol;
use secs_ii::Secs2Message;
use secs_runtime_core::{MachineError, MachineSignal, MessageMachine, RuntimeMessage};

use crate::transport::{
    Rbit, TimeoutTicket, TransactionKey, TransferContext, Wbit,
    error::{SecsMessageConvertError, SecsTransportError},
    secs1::{
        Secs1Message, Secs1MessageHeader,
        config::Secs1TransportConfig,
        protocol::{
            block_transfer::{Secs1BlockTransferEvent, Secs1BlockTransferMachine},
            message::{Secs1MessageMachine, Secs1MessageSignal},
        },
    },
};

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

    fn pump(&mut self) -> Result<(), MachineError> {
        loop {
            let mut progressed = false;

            while let Some(block) = self.block_transfer.poll_read() {
                self.message
                    .handle_read(block)
                    .map_err(Self::map_transport_error)?;
                progressed = true;
            }

            while let Some(event) = self.block_transfer.poll_event() {
                self.handle_block_transfer_event(event)?;
                progressed = true;
            }

            while let Some(block) = self.message.poll_write() {
                self.block_transfer
                    .handle_write(block)
                    .map_err(Self::map_transport_error)?;
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
    ) -> Result<(), MachineError> {
        match event {
            Secs1BlockTransferEvent::SendSuccess { header } => self
                .message
                .handle_event(Secs1MessageSignal::BlockSendSuccess { header })
                .map_err(Self::map_transport_error),
            Secs1BlockTransferEvent::SendFailed { header, .. } => self
                .message
                .handle_event(Secs1MessageSignal::BlockSendFailed { header })
                .map_err(Self::map_transport_error),
            Secs1BlockTransferEvent::ReceiveFailed { error }
            | Secs1BlockTransferEvent::ErrorOccurred(error) => {
                Err(Self::map_transport_error(error))
            }
            Secs1BlockTransferEvent::Retry(_) => Ok(()),
        }
    }

    fn to_secs1_message(&self, msg: RuntimeMessage) -> Secs1Message {
        let stream = msg.payload.stream;
        let function = msg.payload.function;
        let need_reply = msg.payload.need_reply;
        let body = msg.payload.body;
        let system_byte = msg.transaction_key.system_byte;

        let header = Secs1MessageHeader {
            device_id: self.config.device_id,
            rbit: self.send_rbit(function.is_primary()),
            wbit: Wbit(need_reply),
            stream,
            function,
            system_byte,
        };

        Secs1Message::new(header, body)
    }

    fn to_runtime_message(msg: Secs1Message) -> RuntimeMessage {
        let transaction_key = TransactionKey::from(
            TransferContext::Recv,
            msg.header.function.is_primary(),
            msg.header.system_byte,
        );
        let payload = Secs2Message::new(
            msg.header.stream,
            msg.header.function,
            msg.header.wbit.need_reply(),
            msg.body,
        );

        RuntimeMessage::new(transaction_key, payload)
    }

    fn send_rbit(&self, is_primary: bool) -> Rbit {
        match (self.config.local_role.is_active(), is_primary) {
            (true, true) | (false, false) => Rbit::FORWARD,
            (true, false) | (false, true) => Rbit::REVERSE,
        }
    }

    fn map_transport_error(error: SecsTransportError) -> MachineError {
        match error {
            SecsTransportError::SendFailed(key) => MachineError::SendFailed(key),
            SecsTransportError::RecvFailed => MachineError::ReceiveFailed(TransactionKey::new(
                crate::transport::TransactionOwner::Remote,
                crate::transport::SystemByte(0),
            )),
            SecsTransportError::Timeout(_) => MachineError::InvalidTimeout,
            SecsTransportError::MessageConvertFailed(error) => Self::map_convert_error(error),
            SecsTransportError::InvalidState => MachineError::InvalidState,
            SecsTransportError::InvalidBlockHeader
            | SecsTransportError::InvalidBlockLength(_)
            | SecsTransportError::BlockError
            | SecsTransportError::UnexpectedBlock(_)
            | SecsTransportError::UnknownDeviceId(_)
            | SecsTransportError::NoMatchReplyTransaction(_, _)
            | SecsTransportError::NoSuchTransaction(_) => MachineError::InvalidMessage,
            SecsTransportError::ConnectionFailed(_) | SecsTransportError::ConnectionClosed => {
                MachineError::TransportSpecific(1)
            }
        }
    }

    fn map_convert_error(error: SecsMessageConvertError) -> MachineError {
        match error {
            SecsMessageConvertError::EncodeFailed(_) => MachineError::EncodeFailed,
            SecsMessageConvertError::DecodeFailed(_) => MachineError::DecodeFailed,
            SecsMessageConvertError::SequenceGap(_)
            | SecsMessageConvertError::MissingEbit
            | SecsMessageConvertError::EmptyBlocks => MachineError::InvalidMessage,
        }
    }
}

impl MessageMachine for Secs1Machine {
    fn handle_read_bytes(&mut self, bytes: &[u8]) -> Result<(), MachineError> {
        self.block_transfer
            .handle_read(bytes)
            .map_err(Self::map_transport_error)?;
        self.pump()
    }

    fn handle_write_message(&mut self, msg: RuntimeMessage) -> Result<(), MachineError> {
        let msg = self.to_secs1_message(msg);
        self.message
            .handle_write(msg)
            .map_err(Self::map_transport_error)?;
        self.pump()
    }

    fn handle_signal(&mut self, signal: MachineSignal) -> Result<(), MachineError> {
        match signal {
            MachineSignal::LinkOpened | MachineSignal::LinkClosed => Ok(()),
            MachineSignal::WriteCompleted | MachineSignal::WriteFailed => Ok(()),
        }
    }

    fn handle_timeout(&mut self, ticket: TimeoutTicket) -> Result<(), MachineError> {
        match ticket.timeout {
            crate::transport::SecsTimeoutUnit::T1 | crate::transport::SecsTimeoutUnit::T2 => self
                .block_transfer
                .handle_timeout(ticket)
                .map_err(Self::map_transport_error)?,
            crate::transport::SecsTimeoutUnit::T3(_) | crate::transport::SecsTimeoutUnit::T4(_) => {
                self.message
                    .handle_timeout(ticket)
                    .map_err(Self::map_transport_error)?
            }
            _ => {}
        }

        self.pump()
    }

    fn poll_read_message(&mut self) -> Option<RuntimeMessage> {
        let _ = self.pump();
        self.message.poll_read().map(Self::to_runtime_message)
    }

    fn poll_write_bytes(&mut self) -> Option<Vec<u8>> {
        let _ = self.pump();
        self.block_transfer.poll_write()
    }

    fn poll_event(&mut self) -> Option<secs_runtime_core::MachineEvent> {
        None
    }

    fn poll_timeout(&mut self) -> Option<TimeoutTicket> {
        self.block_transfer
            .poll_timeout()
            .or_else(|| self.message.poll_timeout())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use core::time::Duration;
    use secs_ii::{FunctionId, Secs2Message, StreamId, item::Secs2Variant};

    use crate::transport::{
        ConnectionRole, DeviceId, SecsTimeoutUnit, SystemByte, TransactionOwner,
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

    fn build_runtime_primary(system_byte: SystemByte) -> RuntimeMessage {
        let payload = Secs2Message::new(StreamId(1), FunctionId(3), false, Some(build_body()));
        RuntimeMessage::new(
            TransactionKey::new(TransactionOwner::Local, system_byte),
            payload,
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
    fn test_runtime_message_to_bytes() {
        let config = build_config();
        let system_byte = SystemByte(101);
        let mut machine = Secs1Machine::new(config);

        machine
            .handle_write_message(build_runtime_primary(system_byte))
            .unwrap();

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
    fn test_bytes_to_runtime_message() {
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
        assert_eq!(
            msg.transaction_key,
            TransactionKey::new(TransactionOwner::Remote, system_byte)
        );
        assert_eq!(msg.stream(), StreamId(1));
        assert_eq!(msg.function(), FunctionId(3));
        assert!(!msg.need_reply());
        assert!(msg.payload.body.is_some());
    }
}

use alloc::boxed::Box;
use alloc::collections::VecDeque;

use sansio::Protocol;
use secs_common::{SystemByteSource, TimeoutTicket, TransactionKey, TransferContext};
use secs_ii::Secs2Message;
use secs_runtime_core::{
    ByteDataSource, ByteDataSourceError, MachineError, MessageTransport, RuntimeMessage,
};

use crate::{
    transport::{
        SessionId,
        hsms::{
            HsmsControl, HsmsHeader, HsmsMessage, HsmsSType,
            config::HsmsTransportConfig,
            protocol::{
                message::{HsmsMessageEvent, HsmsMessageMachine, HsmsMessageSignal, HsmsWrite},
                session::{HsmsSession, HsmsSessionEffect, HsmsSessionSignal},
            },
        },
    },
    util::time::TimeoutManager,
};

pub mod assembler;
pub mod message;
pub mod session;

pub struct HsmsTransport {
    session: HsmsSession,
    machine: HsmsMessageMachine,
    data_source: Box<dyn ByteDataSource>,

    pending_messages: VecDeque<HsmsMessage>,
    pending_writes: VecDeque<HsmsWrite>,
    outgoing_messages: VecDeque<HsmsMessage>,
    outgoing_timeouts: VecDeque<TimeoutTicket>,
    timeout_manager: TimeoutManager,
    sb_source: SystemByteSource,
    session_id: SessionId,
}

impl HsmsTransport {
    pub fn new(
        config: &HsmsTransportConfig,
        data_source: Box<dyn ByteDataSource>,
        sb_source: SystemByteSource,
    ) -> Self {
        Self {
            session: HsmsSession::new(config.session_id, config.connection_mode),
            machine: HsmsMessageMachine::new(config),
            data_source,
            sb_source,
            pending_messages: VecDeque::new(),
            pending_writes: VecDeque::new(),
            outgoing_messages: VecDeque::new(),
            outgoing_timeouts: VecDeque::new(),
            timeout_manager: TimeoutManager::new(),
            session_id: config.session_id,
        }
    }

    // control을 Hsms 메시지로 변환
    fn control_to_message(&mut self, control: HsmsControl) -> HsmsMessage {
        let system_byte = match control {
            HsmsControl::RejectReq(_, _, system_byte) => system_byte,
            _ => self.sb_source.next_system_byte(),
        };

        let header = match control {
            HsmsControl::SelectReq => HsmsHeader::control(0, 0, HsmsSType::SelectReq, system_byte),
            HsmsControl::SelectRsp(status) => {
                let status: u8 = status.into();
                HsmsHeader::control(0, status, HsmsSType::SelectRsp, system_byte)
            }
            HsmsControl::DeselectReq => {
                HsmsHeader::control(0, 0, HsmsSType::DeselectReq, system_byte)
            }
            HsmsControl::DeselectRsp(status) => {
                let status: u8 = status.into();
                HsmsHeader::control(0, status, HsmsSType::DeselectRsp, system_byte)
            }
            HsmsControl::LinktestReq => {
                HsmsHeader::control(0, 0, HsmsSType::LinktestReq, system_byte)
            }
            HsmsControl::LinktestRsp => {
                HsmsHeader::control(0, 0, HsmsSType::LinktestRsp, system_byte)
            }
            HsmsControl::RejectReq(byte2, reason, system_byte) => {
                let reason: u8 = reason.into();
                HsmsHeader::control(byte2, reason, HsmsSType::RejectReq, system_byte)
            }
            HsmsControl::SeparateReq => {
                HsmsHeader::control(0, 0, HsmsSType::SeparateReq, system_byte)
            }
        };

        HsmsMessage::new(header, None)
    }

    fn runtime_to_message(&self, msg: RuntimeMessage) -> HsmsMessage {
        let system_byte = msg.system_byte();
        let payload = msg.into_payload();
        let header = HsmsHeader::data(
            self.session_id,
            payload.stream,
            payload.function,
            payload.need_reply,
            system_byte,
        );

        HsmsMessage::new(header, payload.body)
    }

    fn message_to_runtime(msg: HsmsMessage) -> RuntimeMessage {
        let header = msg.header;
        let payload = Secs2Message::new(
            header.stream(),
            header.function(),
            header.need_reply(),
            msg.payload,
        );
        let key = TransactionKey::from(
            TransferContext::Recv,
            header.function().is_primary(),
            header.system_byte,
        );

        RuntimeMessage::new(key, payload)
    }

    fn handle_effects(
        &mut self,
        effects: alloc::vec::Vec<HsmsSessionEffect>,
    ) -> Result<(), MachineError> {
        for effect in effects {
            self.handle_effect(effect)?;
        }

        Ok(())
    }

    fn handle_effect(&mut self, effect: HsmsSessionEffect) -> Result<(), MachineError> {
        match effect {
            HsmsSessionEffect::Connect => match self.data_source.open() {
                Ok(()) => {
                    log::debug!("success to open datasource");
                    self.handle_session_signal(HsmsSessionSignal::Connected)?;
                }
                Err(error) => {
                    log::error!("failed to open datasource. reason = {:?}", error);
                    return Err(MachineError::InvalidState);
                }
            },
            HsmsSessionEffect::Disconnect => match self.data_source.close() {
                Ok(()) => {
                    log::debug!("success to close datasource");
                    self.handle_session_signal(HsmsSessionSignal::Disconnected)?;

                    // 공식 명세는 close TCP/IP connection 정도로 표현하나,
                    // 다음 연결을 위해 accept 상태가 되는 것이 자연스러움
                    if self.session.is_passive() {
                        log::debug!("passive transport will wait for next peer");
                        let effects = self.session.connect().map_err(|error| {
                            log::error!(
                                "failed to re-open passive hsms session after disconnect: {:?}",
                                error
                            );
                            MachineError::InvalidState
                        })?;
                        self.handle_effects(effects)?;
                    }
                }
                Err(error) => {
                    log::error!("failed to close datasource. reason = {:?}", error);
                    return Err(MachineError::InvalidState);
                }
            },
            HsmsSessionEffect::SendControl(control) => {
                log::debug!("send hsms control: {:?}", control);
                let msg = self.control_to_message(control);
                self.pending_messages.push_back(msg);
            }
            HsmsSessionEffect::StartTimeout(timeout) => {
                let ticket = self.timeout_manager.issue(timeout);
                self.outgoing_timeouts.push_back(ticket);
            }
            HsmsSessionEffect::ClearTimeout(timeout) => {
                self.timeout_manager.cancel(timeout);
            }
        }

        Ok(())
    }

    fn handle_session_signal(&mut self, signal: HsmsSessionSignal) -> Result<(), MachineError> {
        match self.session.handle(signal) {
            Ok(effects) => self.handle_effects(effects),
            Err(error) => {
                log::error!("hsms session error occured: {:?}", error);
                Err(MachineError::InvalidState)
            }
        }
    }

    fn handle_machine_events(&mut self) -> Result<(), MachineError> {
        while let Some(event) = self.machine.poll_event() {
            match event {
                HsmsMessageEvent::StartTimeout(timeout) => {
                    let ticket = self.timeout_manager.issue(timeout);
                    self.outgoing_timeouts.push_back(ticket);
                }
                HsmsMessageEvent::ClearTimeout(timeout) => {
                    self.timeout_manager.cancel(timeout);
                }
                HsmsMessageEvent::ErrorOccured(error) => {
                    log::error!("hsms message error occured: {:?}", error);
                    return Err(MachineError::InvalidMessage);
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn process_received_messages(&mut self) -> Result<(), MachineError> {
        while let Some(msg) = self.machine.poll_read() {
            if msg.is_control() {
                let control =
                    HsmsControl::try_from(msg).map_err(|_| MachineError::InvalidMessage)?;
                log::debug!("recv hsms control: {:?}", control);
                self.handle_session_signal(HsmsSessionSignal::RecvControl(control))?;
            } else {
                self.outgoing_messages.push_back(msg);
            }
        }

        Ok(())
    }

    fn collect_pending_writes(&mut self) {
        while let Some(write) = self.machine.poll_write() {
            self.pending_writes.push_back(write);
        }
    }

    fn process_pending_messages(&mut self) -> Result<(), MachineError> {
        while let Some(msg) = self.pending_messages.pop_front() {
            self.machine
                .handle_write(msg)
                .map_err(|_| MachineError::EncodeFailed)?;
            self.handle_machine_events()?;
        }

        Ok(())
    }

    fn flush_writes(&mut self) -> Result<(), MachineError> {
        self.collect_pending_writes();

        while let Some(write) = self.pending_writes.pop_front() {
            let header = write.header;
            match self.data_source.write(&write.bytes) {
                Ok(()) => {
                    self.machine
                        .handle_event(HsmsMessageSignal::SendSuccess(header))
                        .map_err(|_| MachineError::InvalidState)?;
                    self.handle_machine_events()?;
                }
                Err(error) if error.is_temporary() => {
                    self.pending_writes.push_front(write);
                    return Ok(());
                }
                Err(error) => {
                    log::error!("failed to write datasource. reason = {:?}", error);
                    let _ = self
                        .machine
                        .handle_event(HsmsMessageSignal::SendFailed(header));
                    let _ = self.handle_session_signal(HsmsSessionSignal::Disconnected);
                    return Err(MachineError::InvalidState);
                }
            }
        }

        Ok(())
    }

    fn poll_source_read(&mut self) -> Result<(), MachineError> {
        if !self.data_source.is_open() {
            log::debug!("source not opened");
            return Ok(());
        }

        let mut buf = [0u8; 4096];
        let len = match self.data_source.read(&mut buf) {
            Ok(len) => len,
            Err(error) if error.is_temporary() => return Ok(()),
            Err(ByteDataSourceError::Disconnected) => {
                self.handle_session_signal(HsmsSessionSignal::Disconnected)?;
                return Ok(());
            }
            Err(error) => {
                log::error!("failed to read datasource. reason = {:?}", error);
                self.handle_session_signal(HsmsSessionSignal::Disconnected)?;
                return Err(MachineError::InvalidState);
            }
        };

        if len == 0 {
            return Ok(());
        }

        self.machine
            .handle_read(&buf[..len])
            .map_err(|_| MachineError::DecodeFailed)?;
        self.handle_machine_events()?;
        self.process_received_messages()
    }

    fn handle_session_timeout(
        &mut self,
        unit: secs_common::SecsTimeoutUnit,
    ) -> Result<(), MachineError> {
        self.handle_session_signal(HsmsSessionSignal::Timeout(unit))
    }

    pub fn linktest(&mut self) -> Result<(), MachineError> {
        match self.session.linktest() {
            Ok(effects) => self.handle_effects(effects),
            Err(error) => {
                log::error!("hsms session error occured: {:?}", error);
                Err(MachineError::InvalidState)
            }
        }
    }

    pub fn separate(&mut self) -> Result<(), MachineError> {
        match self.session.separate() {
            Ok(effects) => self.handle_effects(effects),
            Err(error) => {
                log::error!("hsms session error occured: {:?}", error);
                Err(MachineError::InvalidState)
            }
        }
    }
}

impl MessageTransport for HsmsTransport {
    fn start(&mut self) -> Result<(), MachineError> {
        log::debug!("start hsms transport");
        match self.session.connect() {
            Ok(effects) => self.handle_effects(effects),
            Err(error) => {
                log::error!("hsms session error occured: {:?}", error);
                Err(MachineError::InvalidState)
            }
        }
    }

    fn poll(&mut self) -> Result<(), MachineError> {
        self.poll_source_read()?;
        self.handle_machine_events()?;
        self.process_received_messages()?;
        self.process_pending_messages()?;
        self.handle_machine_events()?;
        self.flush_writes()?;
        self.handle_machine_events()?;
        self.process_received_messages()
    }

    fn send(&mut self, msg: RuntimeMessage) -> Result<(), MachineError> {
        let msg = self.runtime_to_message(msg);
        if !self.session.is_allowed(&msg.header) {
            return Err(MachineError::InvalidState);
        }

        self.pending_messages.push_back(msg);
        Ok(())
    }

    fn poll_recv(&mut self) -> Option<RuntimeMessage> {
        self.outgoing_messages
            .pop_front()
            .map(Self::message_to_runtime)
    }

    fn handle_timeout(&mut self, ticket: TimeoutTicket) -> Result<(), MachineError> {
        let is_timeout_valid = self.timeout_manager.fire(&ticket);

        if !is_timeout_valid {
            return Ok(());
        }

        let unit = ticket.timeout;

        match unit {
            secs_common::SecsTimeoutUnit::T3(..) => self
                .machine
                .handle_timeout(ticket)
                .map_err(|_| MachineError::InvalidState),
            secs_common::SecsTimeoutUnit::T5
            | secs_common::SecsTimeoutUnit::T6
            | secs_common::SecsTimeoutUnit::T7 => self.handle_session_timeout(unit),
            secs_common::SecsTimeoutUnit::T8 => {
                let machine_result = self
                    .machine
                    .handle_timeout(ticket)
                    .map_err(|_| MachineError::InvalidState);

                let session_result = self.handle_session_timeout(unit);

                match (machine_result, session_result) {
                    (Ok(()), Ok(())) => Ok(()),
                    (Err(error), _) => Err(error),
                    (_, Err(error)) => Err(error),
                }
            }
            _ => {
                log::error!("unsupported timeout entered. {:?}", unit);
                Err(MachineError::InvalidTimeout)
            }
        }?;

        self.handle_machine_events()?;
        self.process_received_messages()?;
        self.process_pending_messages()?;
        self.handle_machine_events()?;
        self.flush_writes()
    }

    fn poll_timeout(&mut self) -> Option<TimeoutTicket> {
        self.outgoing_timeouts.pop_front()
    }
}

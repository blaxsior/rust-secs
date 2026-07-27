use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use sansio::Protocol;
use secs_common::TimeoutTicket;
use secs_runtime_core::{ByteDataSource, MachineError, MachineEvent, MessageTransport};

use crate::{
    transport::hsms::{
        HsmsMessage,
        config::HsmsTransportConfig,
        protocol::{
            message::{HsmsMessageMachine, HsmsWrite},
            session::{HsmsSession, HsmsSessionEffect},
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
    source: Box<dyn ByteDataSource>,

    outgoing_writes: Vec<HsmsWrite>,
    outgoing_messages: VecDeque<HsmsMessage>,
    outgoing_events: VecDeque<MachineEvent>,
    outgoing_timeouts: VecDeque<TimeoutTicket>,
    timeout_manager: TimeoutManager,
}

impl HsmsTransport {
    pub fn new(config: &HsmsTransportConfig, source: Box<dyn ByteDataSource>) -> Self {
        Self {
            session: HsmsSession::new(config.session_id, config.connection_mode),
            machine: HsmsMessageMachine::new(config),
            source,
            outgoing_writes: Vec::new(),
            outgoing_messages: VecDeque::new(),
            outgoing_events: VecDeque::new(),
            outgoing_timeouts: VecDeque::new(),
            timeout_manager: TimeoutManager::new(),
        }
    }

    fn handle_effects(&mut self, effects: Vec<HsmsSessionEffect>) {
        for effect in effects {
            self.handle_effect(effect);
        }
    }

    fn handle_effect(&mut self, effect: HsmsSessionEffect) {
        match effect {
            HsmsSessionEffect::Connect => {
                match self.source.open() {
                    Ok(_) => {
                        log::debug!("success to open datasource");
                        let _ = self.session.handle(session::HsmsSessionSignal::Connected);
                    }
                    Err(e) => {
                        log::error!("failed to open datasource. reason = {:?}", e);
                        // let _ = self.session.handle(session::HsmsSessionSignal::Disconnected);
                    }
                }
            }
            HsmsSessionEffect::Disconnect => {
                match self.source.close() {
                    Ok(_) => {
                        log::debug!("success to close datasource");
                        let _ = self
                            .session
                            .handle(session::HsmsSessionSignal::Disconnected);
                    }
                    Err(e) => {
                        log::error!("failed to close datasource. reason = {:?}", e);
                        // let _ = self.session.handle(session::HsmsSessionSignal::Connected);
                    }
                }
            }
            HsmsSessionEffect::SendControl(hsms_control) => {}
            // 티켓 발행 후 처리
            HsmsSessionEffect::StartTimeout(timeout) => {
                let ticket = self.timeout_manager.issue(timeout);
                self.outgoing_timeouts.push_back(ticket);
            }
            HsmsSessionEffect::ClearTimeout(timeout) => {
                self.timeout_manager.cancel(timeout);
            }
        }
    }
}

impl MessageTransport for HsmsTransport {
    fn start(&mut self) {
        // 시작 시 machine 과 연동된 세션 연결
        let result = self.session.connect();
        match result {
            Ok(effects) => {
                self.handle_effects(effects);
            }
            Err(e) => {
                log::error!("error occured {:?}", e);
            }
        }
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

    fn handle_timeout(
        &mut self,
        ticket: TimeoutTicket,
    ) -> Result<(), secs_runtime_core::error::MachineError> {
        let is_timeout_valid = self.timeout_manager.fire(&ticket);

        if !is_timeout_valid {
            // 이미 취소된 타임아웃인 경우 무시
            return Ok(());
        }

        let unit = ticket.timeout;

        match unit {
            secs_common::SecsTimeoutUnit::T3(..) => {
                return self
                    .machine
                    .handle_timeout(ticket)
                    .map_err(|_| MachineError::InvalidState);
            }
            secs_common::SecsTimeoutUnit::T5
            | secs_common::SecsTimeoutUnit::T6
            | secs_common::SecsTimeoutUnit::T7 => {
                return match self
                    .session
                    .handle(session::HsmsSessionSignal::Timeout(unit))
                {
                    Ok(effects) => {
                        self.handle_effects(effects);
                        Ok(())
                    }
                    Err(e) => {
                        log::error!("error occured {:?}", e);
                        Err(MachineError::InvalidState)
                    }
                };
            }
            secs_common::SecsTimeoutUnit::T8 => {
                let machine_result = self
                    .machine
                    .handle_timeout(ticket)
                    .map_err(|_| MachineError::InvalidState);

                let session_result = match self
                    .session
                    .handle(session::HsmsSessionSignal::Timeout(unit))
                {
                    Ok(effects) => {
                        self.handle_effects(effects);
                        Ok(())
                    }
                    Err(e) => {
                        log::error!("error occured {:?}", e);
                        Err(MachineError::InvalidState)
                    }
                };

                match (machine_result, session_result) {
                    (Ok(()), Ok(())) => Ok(()),
                    (Err(err), _) => Err(err),
                    (_, Err(err)) => Err(err),
                }
            }
            _ => {
                log::error!("unsupported timeout entered. {:?}", unit);
                return Err(MachineError::InvalidTimeout);
            }
        }
    }

    fn poll_timeout(&mut self) -> Option<TimeoutTicket> {
        self.outgoing_timeouts.pop_front()
    }
}

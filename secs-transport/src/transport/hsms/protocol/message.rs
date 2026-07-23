pub mod transaction;
pub mod transaction_manager;
use alloc::collections::VecDeque;
use alloc::vec::Vec;

use sansio::Protocol;
use secs_common::TransactionKey;
use secs_common::TransferContext;

use crate::transport::ConnectionRole;
use crate::transport::SecsTimeoutUnit;
use crate::transport::SessionId;
use crate::transport::TimeoutTicket;
use crate::transport::error::SecsTransportError;
use crate::transport::hsms::HsmsHeader;
use crate::transport::hsms::HsmsMessage;
use crate::transport::hsms::config::HsmsTransportConfig;
use crate::transport::hsms::protocol::assembler::HsmsAssembler;
use crate::transport::hsms::protocol::message::transaction::HsmsMessageTransaction;
use crate::transport::hsms::protocol::message::transaction::HsmsTransactionEffect;
use crate::transport::hsms::protocol::message::transaction_manager::HsmsTransactionManager;
use crate::util::time::TimeoutManager;

/// HSMS 전송 계층 대응
pub struct HsmsMessageMachine {
    /// byte to message 조립기
    msg_assembler: HsmsAssembler,
    /// serial 송신할 데이터를 임시로 보관하는 큐. poll_write
    outgoing_buffer: VecDeque<u8>,
    /// hsms 통신을 통해 수신한 블록을 저장하는 큐. poll_read
    outgoing_msgs: VecDeque<HsmsMessage>,
    /// 외부로 타임아웃 시작 알리는 큐. poll_timeout
    outgoing_timeouts: VecDeque<TimeoutTicket>,
    /// 외부로 이벤트 알리는 큐. poll_events
    outgoing_events: VecDeque<HsmsMessageEvent>,
    /// timeout 처리 객체
    timeout_manager: TimeoutManager,
    transaction_manager: HsmsTransactionManager,
}

/// 외부로부터 받는 신호
pub enum HsmsMessageSignal {
    /// 메시지 전송 성공
    SendSuccess(HsmsHeader),
    /// 메시지 전송 실패
    SendFailed(HsmsHeader),
}

/// hsms message 처리 중 발생하는 이벤트
pub enum HsmsMessageEvent {
    /// 메시지를 성공적으로 송신
    SendComplete(TransactionKey),
    /// 메시지를 성공적으로 수신
    RecvComplete(TransactionKey),
    /// 트랜잭션 종료 알림
    TransactionEnd(TransactionKey),
    /// reply 필요
    ReplyRequired(TransactionKey),
    /// 메시지 송수신 중 타임아웃 발생
    MessageTimeout(TransactionKey, SecsTimeoutUnit),
    /// 비정상적인 상태 전이 등 에러가 발생한 경우
    ErrorOccured(SecsTransportError),
}

impl HsmsMessageMachine {
    pub fn new(config: &HsmsTransportConfig) -> Self {
        Self {
            msg_assembler: HsmsAssembler::new(),
            outgoing_buffer: VecDeque::new(),
            outgoing_msgs: VecDeque::new(),
            outgoing_timeouts: VecDeque::new(),
            outgoing_events: VecDeque::new(),
            timeout_manager: TimeoutManager::new(),
            transaction_manager: HsmsTransactionManager::new(),
        }
    }

    /// 외부 시스템에 전송할 메시지를 담는다.
    fn emit_event(&mut self, output: HsmsMessageEvent) {
        self.outgoing_events.push_back(output);
    }

    /// timeout을 시작한다.
    fn start_timeout(&mut self, timeout: SecsTimeoutUnit) {
        log::debug!("[message] start timeout {:?}", timeout);
        let ticket = self.timeout_manager.issue(timeout);
        self.outgoing_timeouts.push_back(ticket);
    }

    /// timeout을 취소한다.
    fn cancel_timeout(&mut self, timeout: SecsTimeoutUnit) {
        log::debug!("[message] cancel timeout {:?}", timeout);
        self.timeout_manager.cancel(timeout);
    }

    /// header로부터 recv transaction_key를 획득
    fn get_transaction_key(context: TransferContext, header: &HsmsHeader) -> TransactionKey {
        let is_primary = header.function().is_primary();
        let system_byte = header.system_byte;

        TransactionKey::from(context, is_primary, system_byte)
    }

    fn take_transaction_outputs(
        transaction: &mut HsmsMessageTransaction,
    ) -> Vec<HsmsTransactionEffect> {
        let mut effects = Vec::new();
        while let Some(effect) = transaction.poll_event() {
            effects.push(effect);
        }
        effects
    }

    fn process_send(&mut self, msg: HsmsMessage) {
        log::debug!(
            "[message] process send: stream={:?} function={:?} need_reply={} system_byte={:?}",
            msg.header.stream(),
            msg.header.function(),
            msg.header.need_reply(),
            msg.header.system_byte
        );

        match msg.to_bytes() {
            Ok(bytes) => {
                self.outgoing_buffer.extend(bytes);
            }
            Err(error) => {
                self.emit_event(HsmsMessageEvent::ErrorOccured(error));
            }
        }

        // 요청 -> 트랜잭션을 새롭게 생성
        let transaction_key = Self::get_transaction_key(TransferContext::Send, &msg.header);
        let transaction = match self
            .transaction_manager
            .create_send(&transaction_key, *msg.header())
        {
            Some(t) => t,
            None => {
                log::warn!(
                    "[message] failed to create send transaction: {:?}",
                    transaction_key
                );
                self.emit_event(HsmsMessageEvent::ErrorOccured(
                    SecsTransportError::NoSuchTransaction(transaction_key),
                ));
                return;
            }
        };
        let outputs = Self::take_transaction_outputs(transaction);
        self.handle_transaction_outputs(outputs, &transaction_key);
    }

    fn process_receive(&mut self, msg: HsmsMessage) {
        log::debug!("[message] process receive: {:?}", msg.header);
        // Routing ERROR: 내가 다루는 deviceId가 아님 -> 에러 알리고 무시
        let transaction_key = Self::get_transaction_key(TransferContext::Recv, &msg.header);

        // 기존 트랜잭션이 있나? 없으면 생성 후 대응
        let transaction = match self.transaction_manager.find(&transaction_key) {
            Some(t) => t,
            None => match self.transaction_manager.create_recv(&transaction_key) {
                Some(t) => t,
                None => unreachable!("unreachable code"),
            },
        };
        transaction.handle_recv(&msg.header);
        let outputs = Self::take_transaction_outputs(transaction);
        self.handle_transaction_outputs(outputs, &transaction_key);

        self.outgoing_msgs.push_back(msg);
    }

    // 외부 signal에 대해 아이템을 처리한다.
    fn process_signal(&mut self, signal: HsmsMessageSignal) {
        log::debug!("[message] process signal");
        // send 처리를 위함
        let header = match signal {
            HsmsMessageSignal::SendSuccess(header) => header,
            HsmsMessageSignal::SendFailed(header) => header,
        };

        // 트랜잭션 찾아오기
        let transaction_key = Self::get_transaction_key(TransferContext::Send, &header);
        let Some(transaction) = self.transaction_manager.find(&transaction_key) else {
            log::debug!("[message] no transaction for signal: {:?}", transaction_key);
            return;
        };

        let _ = transaction.handle_event(signal);
        let outputs = Self::take_transaction_outputs(transaction);
        self.handle_transaction_outputs(outputs, &transaction_key);
    }

    fn handle_transaction_outputs(
        &mut self,
        effects: Vec<HsmsTransactionEffect>,
        transaction_key: &TransactionKey,
    ) {
        for effect in effects {
            log::debug!("[message] handle effect: {:?}", effect);
            match effect {
                HsmsTransactionEffect::StartTimeout(secs_timeout_unit) => {
                    self.start_timeout(secs_timeout_unit);
                }
                HsmsTransactionEffect::ClearTimeout(secs_timeout_unit) => {
                    self.cancel_timeout(secs_timeout_unit);
                }
                HsmsTransactionEffect::ErrorOccured(error) => {
                    if let SecsTransportError::Timeout(timeout_unit) = error {
                        self.emit_event(HsmsMessageEvent::MessageTimeout(
                            *transaction_key,
                            timeout_unit,
                        ));
                    } else {
                        self.emit_event(HsmsMessageEvent::ErrorOccured(error));
                    }
                }
                HsmsTransactionEffect::RecvComplete => {
                    self.emit_event(HsmsMessageEvent::RecvComplete(*transaction_key));
                }
                HsmsTransactionEffect::SendComplete => {
                    self.emit_event(HsmsMessageEvent::SendComplete(*transaction_key));
                }
                HsmsTransactionEffect::TransactionEnd => {
                    self.transaction_manager.remove(transaction_key);
                    self.emit_event(HsmsMessageEvent::TransactionEnd(*transaction_key));
                }
                HsmsTransactionEffect::ReplyRequired => {
                    self.emit_event(HsmsMessageEvent::ReplyRequired(*transaction_key));
                }
            }
        }
    }
}

impl Protocol<&[u8], HsmsMessage, HsmsMessageSignal> for HsmsMessageMachine {
    type Rout = HsmsMessage;
    type Wout = Vec<u8>;
    type Eout = HsmsMessageEvent;
    type Error = SecsTransportError;
    type Time = TimeoutTicket;

    // recv
    fn handle_read(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        // 데이터가 들어왔으므로 t8 timeout 초기화
        self.cancel_timeout(SecsTimeoutUnit::T8);

        self.msg_assembler.handle_read(bytes)?;

        // 생성된 메시지 외부로 전달
        while let Some(msg) = self.msg_assembler.poll_message() {
            self.process_receive(msg);
        }

        // 버퍼에 남은 내용이 있는 경우 timeout 다시 시작
        if self.msg_assembler.has_remained() {
            self.start_timeout(SecsTimeoutUnit::T8);
        }

        Ok(())
    }

    fn poll_read(&mut self) -> Option<Self::Rout> {
        self.outgoing_msgs.pop_front()
    }

    fn handle_write(&mut self, msg: HsmsMessage) -> Result<(), Self::Error> {
        self.process_send(msg);

        Ok(())
    }

    fn poll_write(&mut self) -> Option<Self::Wout> {
        let data: Vec<u8> = self.outgoing_buffer.drain(..).collect();
        if data.is_empty() { None } else { Some(data) }
    }

    /// timeout 발생 시 처리
    fn handle_timeout(&mut self, timeticket: Self::Time) -> Result<(), Self::Error> {
        // 1. _now에서 타임아웃 유닛 추출 (구조체에 맞게 메서드 호출)
        // 예: let expired_unit = now.get_timeout_unit();
        let expired_unit = timeticket.timeout;

        let is_timeout_valid = self.timeout_manager.fire(&timeticket);
        if !is_timeout_valid {
            // 이미 취소된 타임아웃인 경우 -> 무시
            return Ok(());
        }

        // 메시지 전송 중에는 T3 / T8만 다룸
        // T5 / T6 / T7은 세션에서 다룸
        match &expired_unit {
            // send but not received
            SecsTimeoutUnit::T3(key) => {
                let Some(transaction) = self.transaction_manager.find(key) else {
                    return Ok(());
                };

                transaction.handle_timeout(expired_unit).unwrap();
                let outputs = Self::take_transaction_outputs(transaction);
                self.handle_transaction_outputs(outputs, key);
            }
            SecsTimeoutUnit::T8 => {
                // 기존에 있던 데이터를 다 비움
                self.msg_assembler.clear();
                // 외부 엔진에서는 error을 받게 되면 separate로 일괄 처리
                self.emit_event(HsmsMessageEvent::ErrorOccured(
                    SecsTransportError::RecvFailed,
                ));
            }
            _ => {}
        }

        Ok(())
    }

    fn poll_timeout(&mut self) -> Option<Self::Time> {
        self.outgoing_timeouts.pop_front()
    }

    fn handle_event(&mut self, signal: HsmsMessageSignal) -> Result<(), Self::Error> {
        self.process_signal(signal);

        Ok(())
    }

    /// 외부 시스템에서 전송할 메시지를 가져간다.
    fn poll_event(&mut self) -> Option<Self::Eout> {
        self.outgoing_events.pop_front()
    }
}

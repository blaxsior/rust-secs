use crate::transport::error::SecsTransportError;
use crate::transport::hsms::HsmsHeader;
use crate::transport::hsms::protocol::message::HsmsMessageSignal;
use crate::transport::{SecsTimeoutUnit, TransactionKey};

use alloc::collections::VecDeque;
use log;

#[derive(Debug)]
pub enum HsmsTransactionState {
    /// 트랜잭션 생성 초기 대기 상태
    Idle,
    /// 메시지를 전송 중인 상태
    Send,
    /// Primary 전송 후 Secondary Message 대기 중 (T3: reply Timer)
    WaitRecv,
    /// Message 수신 중인 상태 (T4: inter block timer)
    Recv,
    /// 트랜잭션 종료
    End,
}

// state 구현은 transaction에서만 접근 가능
impl HsmsTransactionState {
    fn name(&self) -> &'static str {
        match self {
            Self::Idle => "IDLE",
            Self::Send => "SEND",
            Self::WaitRecv => "WAIT_RECV",
            Self::Recv => "RECV",
            Self::End => "END",
        }
    }

    /// send transaction state 생성
    fn new_send() -> Self {
        Self::Send
    }

    /// recv transaction state 생성
    fn new_recv() -> Self {
        Self::Recv
    }
}

#[derive(Debug, PartialEq, Eq)]
// 트랜잭션에 의한 처리가 필요한 경우
pub enum HsmsTransactionEffect {
    /// Timeout 실행
    StartTimeout(SecsTimeoutUnit),
    /// Timeout 초기화
    ClearTimeout(SecsTimeoutUnit),
    /// 트랜잭션 중 에러 발생
    ErrorOccured(SecsTransportError),
    /// 메시지 송신 성공
    SendComplete,
    /// 메시지 수신 성공
    RecvComplete,

    /// 메시지에 대해 응답이 필요 -> TransactionKey
    ReplyRequired,
    /// 트랜잭션 종료
    TransactionEnd,
}

/// 트랜잭션을 표현하는 객체
pub struct HsmsMessageTransaction {
    /// 트랜잭션을 구분하는 ID 값
    id: TransactionKey,
    /// 트랜잭션 진행 상태
    state: HsmsTransactionState,
    expected_header: Option<HsmsHeader>,
    outgoing_effects: VecDeque<HsmsTransactionEffect>,
}

impl HsmsMessageTransaction {
    fn new(id: TransactionKey) -> Self {
        Self {
            id,
            state: HsmsTransactionState::Idle,
            expected_header: None,
            outgoing_effects: VecDeque::new(),
        }
    }

    pub fn new_send(id: TransactionKey, header: HsmsHeader) -> Self {
        let mut tx = Self::new(id);
        log::debug!("[tx {:?}] create send transaction", tx.id);
        tx.enter_send(&header);

        tx
    }

    pub fn new_recv(id: TransactionKey) -> Self {
        let mut tx = Self::new(id);
        log::debug!("[tx {:?}] create recv transaction", tx.id);
        tx.enter_recv();
        // tx.handle_recv(&header);

        tx
    }

    pub fn handle_recv(&mut self, header: &HsmsHeader) {
        log::debug!("[tx {:?}] handle receive: {:?}", self.id, header);
        // 중복 block 발생 -> discard 후 recv 대기

        // 상태 전이 수행
        if matches!(self.state, HsmsTransactionState::WaitRecv) {
            log::debug!("[tx {:?}] switch wait recv -> recv", self.id);
            self.enter_recv();
        }

        // second message -> reply timer T3 캔슬
        if header.function().is_secondary() {
            log::debug!("[tx {:?}] first recv block, cancel reply timer", self.id);
            self.cancel_reply_timer();
        }

        log::debug!("[tx {:?}] recv complete", self.id);
        self.emit_effect(HsmsTransactionEffect::RecvComplete);

        if header.function().is_primary() && header.need_reply() {
            // 상대방이 보낸 것에 응답이 필요한 경우 알림. 종료는 동일하게
            self.emit_effect(HsmsTransactionEffect::ReplyRequired);
        }

        self.enter_end();
    }

    fn handle_signal(&mut self, signal: HsmsMessageSignal) {
        match signal {
            HsmsMessageSignal::SendSuccess(header) => {
                log::debug!("[tx {:?}] msg send success: {:?}", self.id, header);
                self.on_send(&header);
            }
            HsmsMessageSignal::SendFailed(header) => {
                log::warn!("[tx {:?}] msg send failed {:?}", self.id, header);
                // 전송 실패 알림
                self.emit_effect(HsmsTransactionEffect::ErrorOccured(
                    SecsTransportError::SendFailed(self.id),
                ));
                // 트랜잭션 종료
                self.enter_end();
            }
        }
    }

    fn switch_state(&mut self, state: HsmsTransactionState) {
        log::debug!(
            "[tx {:?}] state changed: {} -> {}",
            self.id,
            self.state.name(),
            state.name()
        );
        self.state = state;
    }

    /// end state로 진입한다.
    fn enter_end(&mut self) {
        self.switch_state(HsmsTransactionState::End);
        self.emit_effect(HsmsTransactionEffect::TransactionEnd);
        log::debug!("[tx {:?}] transaction end", self.id);
    }

    /// wait recv state로 진입한다.
    fn enter_wait_recv(&mut self, header: &HsmsHeader) {
        self.switch_state(HsmsTransactionState::WaitRecv);
        // reply timer 시작
        self.set_reply_timer(header);
        log::debug!("[tx {:?}] enter wait recv", self.id);
    }

    /// send state로 진입한다.
    fn enter_send(&mut self, header: &HsmsHeader) {
        log::debug!("[tx {:?}] enter send", self.id);
        self.expected_header = Some(*header);
        self.switch_state(HsmsTransactionState::new_send());
    }

    /// recv state로 진입한다.
    fn enter_recv(&mut self) {
        log::debug!("[tx {:?}] enter recv", self.id);
        self.switch_state(HsmsTransactionState::new_recv());
    }

    /// send가 발생했을 때 동작을 정의
    fn on_send(&mut self, header: &HsmsHeader) {
        if !matches!(self.state, HsmsTransactionState::Send) {
            log::warn!("on_send can be called only in send state");
            return;
        }

        if let Some(ex_header) = self.expected_header {
            if !(ex_header == *header) {
                log::error!("send header is not match");
                return;
            }
        } else {
            log::warn!("expected header not set");
            return;
        }

        // 내가 보낸 것과 매칭되는지 한번 검사

        self.emit_effect(HsmsTransactionEffect::SendComplete);

        if header.function().is_primary() && header.need_reply() {
            // primary = 내가 보냄 + need reply => 응답 대기
            self.enter_wait_recv(&header);
        } else {
            // secondary 또는 reply가 필요 없는 경우 -> 종료
            self.enter_end();
        }
    }

    /// 외부로 transaction에 의한 effect 알림
    fn emit_effect(&mut self, effect: HsmsTransactionEffect) {
        self.outgoing_effects.push_back(effect);
    }

    /// reply block expected header
    /// complement R bit, same Device ID, Secondary(func + 1), same SystemByte
    /// wbit / ebit / block no 제외 비교
    fn is_expected_header_for_reply(expected: &HsmsHeader, actual: &HsmsHeader) -> bool {
        return expected.stream() == actual.stream()
            && expected.function() == actual.function()
            && expected.session_id == actual.session_id
            && expected.system_byte == actual.system_byte;
    }

    /// reply timer(T3) 지정 + timer 체크 조건 설정
    fn set_reply_timer(&mut self, header: &HsmsHeader) {
        log::debug!("[tx {:?}] start t3", self.id);
        self.emit_effect(HsmsTransactionEffect::StartTimeout(SecsTimeoutUnit::T3(
            self.id,
        )));
        // rbit 반전 + function + 1
        // secondary 설정
        self.expected_header = Some(HsmsHeader {
            byte3: header.function().reply().0,
            ..*header
        });
    }

    /// reply timer(T3) 초기화
    fn cancel_reply_timer(&mut self) {
        log::debug!("[tx {:?}] cancel t3", self.id);
        self.emit_effect(HsmsTransactionEffect::ClearTimeout(SecsTimeoutUnit::T3(
            self.id,
        )));
    }

    fn on_timeout(&mut self, timeout: SecsTimeoutUnit) {
        if matches!(self.state, HsmsTransactionState::End) {
            return;
        }

        log::warn!("[tx {:?}] timeout occurred: {:?}", self.id, timeout);
        self.emit_effect(HsmsTransactionEffect::ErrorOccured(
            SecsTransportError::Timeout(timeout),
        ));
        self.enter_end();
    }

    pub fn handle_event(&mut self, signal: HsmsMessageSignal) -> Result<(), SecsTransportError> {
        self.handle_signal(signal);
        Ok(())
    }

    pub fn poll_event(&mut self) -> Option<HsmsTransactionEffect> {
        self.outgoing_effects.pop_front()
    }

    pub fn handle_timeout(&mut self, now: SecsTimeoutUnit) -> Result<(), SecsTransportError> {
        self.on_timeout(now);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // use secs_ii::{FunctionId, StreamId, item::Secs2Variant};

    use secs_common::SessionId;
    use secs_ii::{FunctionId, StreamId, item::Secs2Variant};

    use crate::transport::{
        SecsTimeoutUnit, SystemByte, TransactionKey, TransactionOwner, error::SecsTransportError, hsms::{
            HsmsHeader, HsmsMessage, protocol::message::{HsmsMessageSignal, transaction::{
                    HsmsMessageTransaction, HsmsTransactionEffect, HsmsTransactionState,
                }},
        },
    };

    fn build_primary_message(
        session_id: SessionId,
        system_byte: SystemByte,
        need_reply: bool,
    ) -> HsmsMessage {
        let stream = StreamId(1);
        let function = FunctionId(3);
        let body = Secs2Variant::list(vec![Secs2Variant::uint4(3001), Secs2Variant::uint4(3002)]);
        let header = HsmsHeader::data(session_id, stream, function, need_reply, system_byte);

        let msg = HsmsMessage::new(header, Some(body));

        msg
    }

    fn build_secondary_message(
        session_id: SessionId,
        system_byte: SystemByte,
        need_reply: bool,
    ) -> HsmsMessage {
        let stream = StreamId(1);
        let function = FunctionId(4);
        let body = Secs2Variant::list(vec![Secs2Variant::uint8(1500), Secs2Variant::uint8(1501)]);
        let header = HsmsHeader::data(session_id, stream, function, need_reply, system_byte);
        let msg = HsmsMessage::new(header, Some(body));

        msg
    }

    fn drain_effects(transaction: &mut HsmsMessageTransaction) -> Vec<HsmsTransactionEffect> {
        let mut effects = Vec::new();
        while let Some(effect) = transaction.poll_event() {
            effects.push(effect);
        }
        effects
    }

    /// primary send 후 secondary를 요구하는 경우
    #[test]
    fn test_send_primary_need_reply() {
        let session_id = SessionId(10);
        let system_byte = SystemByte(101);

        let msg = build_primary_message(session_id, system_byte, true);

        // 내가 주인인 케이스
        let owner = TransactionOwner::Local;
        let key = TransactionKey::new(owner, system_byte);

        let mut transaction = HsmsMessageTransaction::new_send(key, *msg.header());

        let signal = HsmsMessageSignal::SendSuccess(msg.header);
        transaction.handle_signal(signal);

        // send 후 recv 대기 시 reply timer 활성화
        let effects = drain_effects(&mut transaction);
        // 트랜잭션 종료 판정 X. recv 필요
        assert!(!effects.contains(&HsmsTransactionEffect::TransactionEnd));
        assert!(effects.contains(&HsmsTransactionEffect::SendComplete));
        assert!(
            effects.contains(&HsmsTransactionEffect::StartTimeout(SecsTimeoutUnit::T3(
                key
            )))
        );

        assert!(matches!(transaction.state, HsmsTransactionState::WaitRecv));

        let msg: HsmsMessage = build_secondary_message(session_id, system_byte, false);
        transaction.handle_recv(&msg.header);

        let effects = drain_effects(&mut transaction);
        assert!(effects.contains(&HsmsTransactionEffect::RecvComplete));
        assert!(effects.contains(&HsmsTransactionEffect::TransactionEnd));

        assert!(matches!(transaction.state, HsmsTransactionState::End));
    }

    /// primary send 후 secondary를 요구하지 않는 경우
    #[test]
    fn test_send_primary_no_reply() {
        let session_id = SessionId(10);
        let system_byte = SystemByte(101);

        let msg = build_primary_message(session_id, system_byte, false);

        // 내가 주인인 케이스
        let owner = TransactionOwner::Local;
        let key = TransactionKey::new(owner, system_byte);

        let mut transaction = HsmsMessageTransaction::new_send(key, *msg.header());

        let signal = HsmsMessageSignal::SendSuccess(msg.header);
        transaction.handle_signal(signal);

        // send 후 대기 X -> complete 처리
        let effects = drain_effects(&mut transaction);

        // send complete + transaction end effect 발생
        assert!(effects.contains(&HsmsTransactionEffect::SendComplete));
        assert!(effects.contains(&HsmsTransactionEffect::TransactionEnd));

        assert!(matches!(transaction.state, HsmsTransactionState::End));
    }

    /// primary send 후 reply 대기 중 T3 timeout이 발생하는 경우
    #[test]
    fn test_send_primary_timeout_t3() {
        let session_id = SessionId(10);
        let system_byte = SystemByte(101);

        let msg = build_primary_message(session_id, system_byte, true);

        let owner = TransactionOwner::Local;
        let key = TransactionKey::new(owner, system_byte);

        let mut transaction = HsmsMessageTransaction::new_send(key, *msg.header());

        transaction.handle_signal(HsmsMessageSignal::SendSuccess(msg.header));
        let _ = drain_effects(&mut transaction);

        transaction
            .handle_timeout(SecsTimeoutUnit::T3(key))
            .unwrap();

        let effects = drain_effects(&mut transaction);
        assert!(effects.contains(&HsmsTransactionEffect::ErrorOccured(
            SecsTransportError::Timeout(SecsTimeoutUnit::T3(key))
        )));
        assert!(effects.contains(&HsmsTransactionEffect::TransactionEnd));
        assert!(matches!(transaction.state, HsmsTransactionState::End));
    }

    /// primary recv 후 secondary를 보내야 하는 경우
    #[test]
    fn test_recv_primary_need_reply() {
        let session_id = SessionId(10);
        let system_byte = SystemByte(101);

        // Remote -> Local primary
        let msg = build_primary_message(session_id, system_byte, true);

        // 내가 응답해야 하는 케이스
        let owner = TransactionOwner::Remote;
        let key = TransactionKey::new(owner, system_byte);

        let mut transaction = HsmsMessageTransaction::new_recv(key);
        transaction.handle_recv(&msg.header);
        let effects = drain_effects(&mut transaction);

        assert!(effects.contains(&HsmsTransactionEffect::RecvComplete));
        assert!(effects.contains(&HsmsTransactionEffect::ReplyRequired));
        assert!(effects.contains(&HsmsTransactionEffect::TransactionEnd));
        assert!(matches!(transaction.state, HsmsTransactionState::End));
    }

    // 상대방이 블록 던지고, 응답은 필요 없는 경우
    #[test]
    fn test_recv_primary_no_reply() {
        let session_id = SessionId(10);
        let system_byte = SystemByte(101);

        // Remote -> Local primary
        let msg = build_primary_message(session_id, system_byte, false);

        // 상대방이 트랜잭션 주도
        let owner = TransactionOwner::Remote;
        let key = TransactionKey::new(owner, system_byte);

        let mut transaction = HsmsMessageTransaction::new_recv(key);
        transaction.handle_recv(&msg.header);
        let effects = drain_effects(&mut transaction);

        // 수신 성공
        assert!(effects.contains(&HsmsTransactionEffect::RecvComplete));
        assert!(effects.contains(&HsmsTransactionEffect::TransactionEnd));
        // reply 모드 전이 X
        assert!(
            !effects
                .iter()
                .any(|effect| { matches!(effect, HsmsTransactionEffect::ReplyRequired) })
        );

        assert!(matches!(transaction.state, HsmsTransactionState::End));
    }
}

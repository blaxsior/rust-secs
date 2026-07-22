use crate::transport::error::SecsTransportError;
use crate::transport::secs1::block::Secs1Block;
use crate::transport::secs1::convert::{decode, encode};
use crate::transport::secs1::Secs1Message;
use crate::transport::secs1::{block::Secs1BlockHeader, protocol::message::Secs1MessageSignal};
use crate::transport::{SecsTimeoutUnit, TransactionKey};

use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::mem;
use log;
use sansio::Protocol;
use secs_ii::{FunctionId};

#[derive(Debug)]
pub enum Secs1TransactionState {
    /// 트랜잭션 생성 초기 대기 상태
    Idle,

    /// 메시지 전송 
    Send,

    /// Message 수신
    Recv,

    /// Primary 전송 후 Secondary Message 대기 중 (T3: reply Timer)
    WaitRecv,

    /// Primary 수신 후 Secondary Message 전송 대기 중
    WaitSend,

    /// 트랜잭션 종료
    End,
}

// state 구현은 transaction에서만 접근 가능
impl Secs1TransactionState {
    fn name(&self) -> &'static str {
        match self {
            Self::Idle => "IDLE",
            Self::Send { .. } => "SEND",
            Self::WaitRecv => "WAIT_RECV",
            Self::WaitSend => "WAIT_SEND",
            Self::Recv { .. } => "RECV",
            Self::End => "END",
        }
    }

    /// send transaction state 생성
    fn new_send<B>(blocks: B) -> Self
    where
        B: Into<VecDeque<Secs1Block>>,
    {
        Self::Send {
            blocks: blocks.into(),
        }
    }

    /// recv transaction state 생성
    fn new_recv() -> Self {
        Self::Recv { blocks: Vec::new() }
    }

    /// send 모드에서 block 송신
    fn send_next(&mut self) -> Option<Secs1Block> {
        // Send 상태 -> 마지막 블록 전송
        if let Secs1TransactionState::Send { blocks } = self {
            return blocks.pop_front();
        } else {
            log::warn!("cannot send next block. current state: {:?}", self);
            None
        }
    }

    /// recv 모드에서 block 수신
    fn recv_next(&mut self, block: Secs1Block) {
        if let Secs1TransactionState::Recv { blocks } = self {
            blocks.push(block);
        } else {
            log::warn!("cannot recv next block. current state: {:?}", self);
        }
    }

    fn is_send_end(&self) -> bool {
        match self {
            Self::Send { blocks } => blocks.is_empty(),
            _ => false,
        }
    }

    // 로직 상 last block 내용을 peek 해서 검사하므로 크게 필요 없음
    // fn is_recv_end(&self) -> bool {
    //     match self {
    //         Self::Recv { blocks } => blocks.last().is_some_and(|b| b.header.is_end()),
    //         _ => false,
    //     }
    // }
}

#[derive(Debug, PartialEq, Eq)]
// 트랜잭션에 의한 처리가 필요한 경우
pub enum Secs1TransactionEffect {
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
    ReplyRequired(TransactionKey),
    /// 트랜잭션 종료
    TransactionEnd,
}

/// 트랜잭션을 표현하는 객체
pub struct Secs1MessageTransaction {
    /// 트랜잭션을 구분하는 ID 값
    id: TransactionKey,
    /// 트랜잭션 진행 상태
    state: Secs1TransactionState,
    last_header: Option<Secs1BlockHeader>,
    expected_header: Option<Secs1BlockHeader>,
    outgoing_effects: VecDeque<Secs1TransactionEffect>,
    outgoing_recv_queue: VecDeque<Secs1Message>,
    outgoing_send_queue: VecDeque<Secs1Block>,
}

impl Secs1MessageTransaction {
    fn new(id: TransactionKey) -> Self {
        Self {
            id,
            state: Secs1TransactionState::Idle,
            last_header: None,
            expected_header: None,
            outgoing_effects: VecDeque::new(),
            outgoing_recv_queue: VecDeque::new(),
            outgoing_send_queue: VecDeque::new(),
        }
    }

    pub fn new_send(id: TransactionKey, msg: Secs1Message) -> Self {
        let mut tx = Self::new(id);
        log::debug!("[tx {:?}] create send transaction", tx.id);
        let blocks = match encode(msg) {
            Ok(blocks) => blocks,
            Err(err) => {
                log::warn!("[tx {:?}] message encode failed: {:?}", tx.id, err);
                tx.emit_effect(Secs1TransactionEffect::ErrorOccured(
                    SecsTransportError::MessageConvertFailed(err),
                ));
                tx.enter_end();
                return tx;
            }
        };
        tx.enter_send(blocks);

        tx
    }

    pub fn new_recv(id: TransactionKey) -> Self {
        let mut tx = Self::new(id);
        log::debug!("[tx {:?}] create recv transaction", tx.id);
        tx.enter_recv();

        tx
    }

    fn handle_receive(&mut self, block: Secs1Block) {
        log::debug!("[tx {:?}] handle receive: {:?}", self.id, block.header);
        // 중복 block 발생 -> discard 후 recv 대기
        if self.is_duplicate(&block.header) {
            log::debug!("[tx {:?}] duplicate block ignored", self.id);
            return;
        }
        // 정상적인 블록 -> dup check를 위한 헤더 저장
        self.last_header = Some(block.header);

        // 기대한 블록이 맞는지 검사
        if !self.is_expected(&block.header) {
            // primary first 아님 -> Block Error 발생
            if !block.header.is_primary() || !block.header.is_first_block() {
                log::warn!(
                    "[tx {:?}] unexpected block received: {:?}",
                    self.id,
                    block.header
                );
                self.emit_effect(Secs1TransactionEffect::ErrorOccured(
                    SecsTransportError::UnexpectedBlock(block.header),
                ));
                return;
            }
        }

        // 상태 전이 수행
        if matches!(self.state, Secs1TransactionState::WaitRecv) {
            log::debug!("[tx {:?}] switch wait recv -> recv", self.id);
            self.enter_recv();
        }

        self.state.recv_next(block);

        let Secs1TransactionState::Recv { blocks } = &self.state else {
            return;
        };

        let block = match blocks.last() {
            Some(b) => b,
            None => return,
        };

        let header = block.header;

        // timer 초기화
        if header.is_first_block() {
            // secondary first: cancel reply timer -> wait recv 상태
            if header.is_secondary() {
                log::debug!("[tx {:?}] first recv block, cancel reply timer", self.id);
                self.cancel_reply_timer();
            }
        } else {
            // not first: cancel inter block timer
            log::debug!("[tx {:?}] inter block received, cancel t4", self.id);
            self.cancel_inter_block_timer();
        }

        if header.is_end() {
            // recv 성공, 블록 상위 전송
            let blocks = match &mut self.state {
                Secs1TransactionState::Recv { blocks } => mem::take(blocks),
                _ => unreachable!(),
            };

            let decode_result = decode(blocks);
            match decode_result {
                Ok(msg) => {
                    log::debug!("[tx {:?}] recv complete", self.id);
                    self.write_recv(msg);
                    self.emit_effect(Secs1TransactionEffect::RecvComplete);

                    if header.is_primary() && header.need_reply() {
                        // 상대방이 보낸 것에 응답이 필요한 경우 -> 종료 X, 내부적인 send 대기
                        self.enter_wait_send();
                    } else {
                        self.enter_end();
                    }
                }
                Err(err) => {
                    log::warn!("[tx {:?}] message decode failed: {:?}", self.id, err);
                    // 실패와 함께 트랜잭션 종료
                    self.emit_effect(Secs1TransactionEffect::ErrorOccured(
                        SecsTransportError::MessageConvertFailed(err),
                    ));
                    self.enter_end();
                }
            }
        } else {
            self.set_inter_block_timer(&header);
        }
    }

    fn handle_signal(&mut self, signal: Secs1MessageSignal) {
        match signal {
            Secs1MessageSignal::BlockSendSuccess { header } => {
                log::debug!("[tx {:?}] block send success: {:?}", self.id, header);
                self.on_send(&header);
            }
            Secs1MessageSignal::BlockSendFailed { .. } => {
                log::warn!("[tx {:?}] block send failed", self.id);
                // 전송 실패 알림
                self.emit_effect(Secs1TransactionEffect::ErrorOccured(
                    SecsTransportError::SendFailed(self.id),
                ));
                // 트랜잭션 종료
                self.enter_end();
            }
        }
    }

    fn handle_reply(&mut self, msg: Secs1Message) {
        // 초기 상태 가드
        if !matches!(self.state, Secs1TransactionState::WaitSend) {
            log::warn!("handle_reply works only in wait send state");
            return;
        }

        log::debug!("[tx {:?}] handle reply", self.id);
        let blocks = match encode(msg) {
            Ok(blocks) => blocks,
            Err(err) => {
                log::warn!("[tx {:?}] reply encode failed: {:?}", self.id, err);
                self.emit_effect(Secs1TransactionEffect::ErrorOccured(
                    SecsTransportError::MessageConvertFailed(err),
                ));
                self.enter_end();
                return;
            }
        };
        self.enter_send(blocks);
    }

    fn send_next(&mut self) {
        log::debug!("[tx {:?}] send next block", self.id);
        let result = self.state.send_next();

        if let Some(block) = result {
            self.last_header = Some(block.header);
            self.write_send(block);
        }
    }

    fn write_recv(&mut self, msg: Secs1Message) {
        self.outgoing_recv_queue.push_back(msg);
    }

    fn poll_recv(&mut self) -> Option<Secs1Message> {
        self.outgoing_recv_queue.pop_front()
    }

    fn write_send(&mut self, block: Secs1Block) {
        self.outgoing_send_queue.push_back(block);
    }

    fn poll_send(&mut self) -> Option<Secs1Block> {
        self.outgoing_send_queue.pop_front()
    }

    fn switch_state(&mut self, state: Secs1TransactionState) {
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
        self.switch_state(Secs1TransactionState::End);
        self.emit_effect(Secs1TransactionEffect::TransactionEnd);
        log::debug!("[tx {:?}] transaction end", self.id);
    }

    /// wait send state로 진입한다.
    fn enter_wait_send(&mut self) {
        // 
        self.switch_state(Secs1TransactionState::WaitSend);
        self.emit_effect(Secs1TransactionEffect::ReplyRequired(self.id));
        log::debug!("[tx {:?}] enter wait send", self.id);
    }

    /// wait recv state로 진입한다.
    fn enter_wait_recv(&mut self, header: &Secs1BlockHeader) {
        self.switch_state(Secs1TransactionState::WaitRecv);
        // reply timer 시작
        self.set_reply_timer(header);
        log::debug!("[tx {:?}] enter wait recv", self.id);
    }

    /// send state로 진입한다.
    fn enter_send(&mut self, blocks: Vec<Secs1Block>) {
        log::debug!(
            "[tx {:?}] enter send with {} block(s)",
            self.id,
            blocks.len()
        );
        self.switch_state(Secs1TransactionState::new_send(blocks));
        self.send_next();
    }

    /// recv state로 진입한다.
    fn enter_recv(&mut self) {
        log::debug!("[tx {:?}] enter recv", self.id);
        self.switch_state(Secs1TransactionState::new_recv());
    }

    /// send가 발생했을 때 동작을 정의
    fn on_send(&mut self, header: &Secs1BlockHeader) {
        if !matches!(self.state, Secs1TransactionState::Send { .. }) {
            log::warn!("on_send can be called only in send state");
            return;
        }

        // 블록이 비어 있는 경우
        if self.state.is_send_end() {
            log::debug!("[tx {:?}] message send success", self.id);
            self.emit_effect(Secs1TransactionEffect::SendComplete);

            if header.is_primary() && header.need_reply() {
                // primary = 내가 보냄 + need reply => 응답 대기
                self.enter_wait_recv(&header);
            } else {
                // secondary 또는 reply가 필요 없는 경우 -> 종료
                self.enter_end();
            }
        } else {
            // 블록이 남아 있으면 다음 블록 전송
            self.send_next();
        }
    }

    /// 외부로 transaction에 의한 effect 알림
    fn emit_effect(&mut self, effect: Secs1TransactionEffect) {
        self.outgoing_effects.push_back(effect);
    }

    /// 기대한 블록이 맞는지 체크
    fn is_expected(&self, actual: &Secs1BlockHeader) -> bool {
        let Some(expected) = &self.expected_header else {
            return false;
        };

        match self.state {
            Secs1TransactionState::Recv { .. } => {
                Self::is_expected_header_for_inter_block(expected, actual)
            }
            Secs1TransactionState::WaitRecv => Self::is_expected_header_for_reply(expected, actual),
            _ => false,
        }
    }

    /// inter block expected header
    /// e bit 제외 모두 비교 필요
    fn is_expected_header_for_inter_block(
        expected: &Secs1BlockHeader,
        actual: &Secs1BlockHeader,
    ) -> bool {
        return expected.block_no == actual.block_no
            && expected.rbit == actual.rbit
            && expected.wbit == actual.wbit
            && expected.stream == actual.stream
            && expected.function == actual.function
            && expected.device_id == actual.device_id
            && expected.system_byte == actual.system_byte;
    }

    /// reply block expected header
    /// complement R bit, same Device ID, Secondary(func + 1), same SystemByte
    /// wbit / ebit / block no 제외 비교
    fn is_expected_header_for_reply(
        expected: &Secs1BlockHeader,
        actual: &Secs1BlockHeader,
    ) -> bool {
        return expected.rbit == actual.rbit
            && expected.stream == actual.stream
            && expected.function == actual.function
            && expected.device_id == actual.device_id
            && expected.system_byte == actual.system_byte;
    }

    /// inter block timer(T4) 지정 + timer 체크 조건 설정
    fn set_inter_block_timer(&mut self, header: &Secs1BlockHeader) {
        log::debug!("[tx {:?}] start t4", self.id);
        self.emit_effect(Secs1TransactionEffect::StartTimeout(SecsTimeoutUnit::T4(
            self.id,
        )));
        // t3 timeout 설정 로직 처리 + expected header 갱신
        // block_no만 + 1
        self.expected_header = Some(Secs1BlockHeader {
            block_no: header.block_no + 1,
            ..*header
        });
    }

    /// inter block timer(T4) 초기화
    fn cancel_inter_block_timer(&mut self) {
        log::debug!("[tx {:?}] cancel t4", self.id);
        self.emit_effect(Secs1TransactionEffect::ClearTimeout(SecsTimeoutUnit::T4(
            self.id,
        )));
    }

    /// reply timer(T3) 지정 + timer 체크 조건 설정
    fn set_reply_timer(&mut self, header: &Secs1BlockHeader) {
        log::debug!("[tx {:?}] start t3", self.id);
        self.emit_effect(Secs1TransactionEffect::StartTimeout(SecsTimeoutUnit::T3(
            self.id,
        )));
        // rbit 반전 + function + 1
        // secondary 설정
        self.expected_header = Some(Secs1BlockHeader {
            rbit: header.rbit.complement(),
            function: FunctionId(header.function.0 + 1),
            ..*header
        });
    }

    /// reply timer(T3) 초기화
    fn cancel_reply_timer(&mut self) {
        log::debug!("[tx {:?}] cancel t3", self.id);
        self.emit_effect(Secs1TransactionEffect::ClearTimeout(SecsTimeoutUnit::T3(
            self.id,
        )));
    }

    // /// 헤더가 이전에 수신한 헤더와 동일한지 체크 (중복 수신 방지)
    fn is_duplicate(&self, header: &Secs1BlockHeader) -> bool {
        if let Some(last) = &self.last_header {
            last == header
        } else {
            false
        }
    }

    pub fn get_last_header(&self) -> Option<Secs1BlockHeader> {
        self.last_header
    }

    fn on_timeout(&mut self, timeout: SecsTimeoutUnit) {
        if matches!(self.state, Secs1TransactionState::End) {
            return;
        }

        log::warn!("[tx {:?}] timeout occurred: {:?}", self.id, timeout);
        self.emit_effect(Secs1TransactionEffect::ErrorOccured(
            SecsTransportError::Timeout(timeout),
        ));
        self.enter_end();
    }
}

impl Protocol<Secs1Block, Secs1Message, Secs1MessageSignal> for Secs1MessageTransaction {
    type Rout = Secs1Message;
    type Wout = Secs1Block;
    type Eout = Secs1TransactionEffect;
    type Error = SecsTransportError;
    type Time = SecsTimeoutUnit;

    fn handle_read(&mut self, msg: Secs1Block) -> Result<(), Self::Error> {
        self.handle_receive(msg);
        Ok(())
    }

    fn poll_read(&mut self) -> Option<Self::Rout> {
        self.poll_recv()
    }

    fn handle_write(&mut self, msg: Secs1Message) -> Result<(), Self::Error> {
        self.handle_reply(msg);
        Ok(())
    }

    fn poll_write(&mut self) -> Option<Self::Wout> {
        self.poll_send()
    }

    fn handle_event(&mut self, evt: Secs1MessageSignal) -> Result<(), Self::Error> {
        self.handle_signal(evt);
        Ok(())
    }

    fn poll_event(&mut self) -> Option<Self::Eout> {
        self.outgoing_effects.pop_front()
    }

    fn handle_timeout(&mut self, now: Self::Time) -> Result<(), Self::Error> {
        self.on_timeout(now);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // use secs_ii::{FunctionId, StreamId, item::Secs2Variant};

    use std::collections::VecDeque;

    use secs_ii::{item::Secs2Variant, FunctionId, StreamId};

    use crate::transport::secs1::Secs1Message;
    use crate::{
        transport::error::SecsTransportError,
        transport::{
            secs1::{
                convert::encode,
                protocol::message::{
                    transaction::{
                        Secs1MessageTransaction, Secs1TransactionEffect, Secs1TransactionState,
                    },
                    Secs1MessageSignal,
                },
                Secs1MessageHeader,
            },
            DeviceId, Rbit, SecsTimeoutUnit, SystemByte, TransactionKey, TransactionOwner, Wbit,
        },
    };
    use sansio::Protocol;

    fn build_primary_message(
        device_id: DeviceId,
        system_byte: SystemByte,
        rbit: Rbit,
        need_reply: bool,
    ) -> Secs1Message {
        let stream = StreamId(1);
        let function = FunctionId(3);
        let body = Secs2Variant::list(vec![Secs2Variant::uint4(3001), Secs2Variant::uint4(3002)]);
        let header = Secs1MessageHeader {
            device_id,
            rbit,
            wbit: Wbit(need_reply),
            stream,
            function,
            system_byte,
        };
        let msg = Secs1Message::new(header, Some(body));

        msg
    }

    fn build_secondary_message(
        device_id: DeviceId,
        system_byte: SystemByte,
        rbit: Rbit,
        need_reply: bool,
    ) -> Secs1Message {
        let stream = StreamId(1);
        let function = FunctionId(4);
        let body = Secs2Variant::list(vec![Secs2Variant::uint8(1500), Secs2Variant::uint8(1501)]);
        let header = Secs1MessageHeader {
            device_id,
            rbit,
            wbit: Wbit(need_reply),
            stream,
            function,
            system_byte,
        };
        let msg = Secs1Message::new(header, Some(body));

        msg
    }

    fn drain_effects(transaction: &mut Secs1MessageTransaction) -> Vec<Secs1TransactionEffect> {
        let mut effects = Vec::new();
        while let Some(effect) = transaction.poll_event() {
            effects.push(effect);
        }
        effects
    }

    /// primary send 후 secondary를 요구하는 경우
    #[test]
    fn test_send_primary_need_reply() {
        let device_id = DeviceId(10);
        let system_byte = SystemByte(101);

        let msg = build_primary_message(device_id, system_byte, Rbit::FORWARD, true);

        // 내가 주인인 케이스
        let owner = TransactionOwner::Local;
        let key = TransactionKey::new(owner, system_byte);

        let mut transaction = Secs1MessageTransaction::new_send(key, msg);

        let block = transaction.poll_write().unwrap();

        let signal = Secs1MessageSignal::BlockSendSuccess {
            header: block.header,
        };
        transaction.handle_signal(signal);

        // send 후 recv 대기 시 reply timer 활성화
        let effects = drain_effects(&mut transaction);
        // 트랜잭션 종료 판정 X. recv 필요
        assert!(!effects.contains(&Secs1TransactionEffect::TransactionEnd));
        assert!(effects.contains(&Secs1TransactionEffect::SendComplete));
        assert!(
            effects.contains(&Secs1TransactionEffect::StartTimeout(SecsTimeoutUnit::T3(
                key
            )))
        );

        assert!(matches!(transaction.state, Secs1TransactionState::WaitRecv));

        let msg: Secs1Message =
            build_secondary_message(device_id, system_byte, Rbit::REVERSE, false);
        let expected_dev_id = msg.header.device_id;
        let expected_rbit = msg.header.rbit;
        let expected_system_byte = msg.header.system_byte;
        let expected_stream = msg.header.stream;
        let expected_function = msg.header.function;
        let expected_wbit = msg.header.wbit;

        let mut recv_blocks = VecDeque::from(encode(msg).unwrap());

        transaction.handle_receive(recv_blocks.pop_front().unwrap());

        let effects = drain_effects(&mut transaction);
        assert!(effects.contains(&Secs1TransactionEffect::RecvComplete));
        assert!(effects.contains(&Secs1TransactionEffect::TransactionEnd));

        let result_msg = transaction.poll_read().unwrap();

        assert_eq!(result_msg.header.device_id, expected_dev_id);
        assert_eq!(result_msg.header.rbit, expected_rbit);
        assert_eq!(result_msg.header.system_byte, expected_system_byte);
        assert_eq!(result_msg.header.stream, expected_stream);
        assert_eq!(result_msg.header.function, expected_function);
        assert_eq!(result_msg.header.wbit, expected_wbit);

        assert!(matches!(transaction.state, Secs1TransactionState::End));
    }

    /// primary send 후 secondary를 요구하지 않는 경우
    #[test]
    fn test_send_primary_no_reply() {
        let device_id = DeviceId(10);
        let system_byte = SystemByte(101);

        let msg = build_primary_message(device_id, system_byte, Rbit::FORWARD, false);

        // 내가 주인인 케이스
        let owner = TransactionOwner::Local;
        let key = TransactionKey::new(owner, system_byte);

        let mut transaction = Secs1MessageTransaction::new_send(key, msg);

        let block = transaction.poll_write().unwrap();

        let signal = Secs1MessageSignal::BlockSendSuccess {
            header: block.header,
        };
        transaction.handle_signal(signal);

        // send 후 대기 X -> complete 처리
        let effects = drain_effects(&mut transaction);

        // send complete + transaction end effect 발생
        assert!(effects.contains(&Secs1TransactionEffect::SendComplete));
        assert!(effects.contains(&Secs1TransactionEffect::TransactionEnd));

        assert!(matches!(transaction.state, Secs1TransactionState::End));
    }

    /// primary send 후 reply 대기 중 T3 timeout이 발생하는 경우
    #[test]
    fn test_send_primary_timeout_t3() {
        let device_id = DeviceId(10);
        let system_byte = SystemByte(101);

        let msg = build_primary_message(device_id, system_byte, Rbit::FORWARD, true);

        let owner = TransactionOwner::Local;
        let key = TransactionKey::new(owner, system_byte);

        let mut transaction = Secs1MessageTransaction::new_send(key, msg);
        let block = transaction.poll_write().unwrap();

        transaction.handle_signal(Secs1MessageSignal::BlockSendSuccess {
            header: block.header,
        });
        let _ = drain_effects(&mut transaction);

        transaction
            .handle_timeout(SecsTimeoutUnit::T3(key))
            .unwrap();

        let effects = drain_effects(&mut transaction);
        assert!(effects.contains(&Secs1TransactionEffect::ErrorOccured(
            SecsTransportError::Timeout(SecsTimeoutUnit::T3(key))
        )));
        assert!(effects.contains(&Secs1TransactionEffect::TransactionEnd));
        assert!(matches!(transaction.state, Secs1TransactionState::End));
    }

    /// primary recv 중 inter block 대기 상태에서 T4 timeout이 발생하는 경우
    #[test]
    fn test_recv_primary_timeout_t4() {
        let device_id = DeviceId(10);
        let system_byte = SystemByte(202);

        let large_body = Secs2Variant::list(vec![
            Secs2Variant::uint8(1001),
            Secs2Variant::uint8(1002),
            Secs2Variant::uint8(1003),
            Secs2Variant::uint8(1004),
            Secs2Variant::uint8(1005),
            Secs2Variant::uint8(1006),
            Secs2Variant::uint8(1007),
            Secs2Variant::uint8(1008),
            Secs2Variant::uint8(1009),
            Secs2Variant::uint8(1010),
            Secs2Variant::uint8(2001),
            Secs2Variant::uint8(2002),
            Secs2Variant::uint8(2003),
            Secs2Variant::uint8(2004),
            Secs2Variant::uint8(2005),
            Secs2Variant::uint8(2006),
            Secs2Variant::uint8(2007),
            Secs2Variant::uint8(2008),
            Secs2Variant::uint8(2009),
            Secs2Variant::uint8(2010),
            Secs2Variant::uint8(3001),
            Secs2Variant::uint8(3002),
            Secs2Variant::uint8(3003),
            Secs2Variant::uint8(3004),
            Secs2Variant::uint8(3005),
            Secs2Variant::uint8(3006),
            Secs2Variant::uint8(3007),
            Secs2Variant::uint8(3008),
            Secs2Variant::uint8(3009),
            Secs2Variant::uint8(3010),
        ]);
        let msg = Secs1Message::new(
            Secs1MessageHeader {
                device_id,
                rbit: Rbit::REVERSE,
                wbit: Wbit(true),
                stream: StreamId(1),
                function: FunctionId(3),
                system_byte,
            },
            Some(large_body),
        );

        let recv_blocks = encode(msg).unwrap();

        let owner = TransactionOwner::Remote;
        let key = TransactionKey::new(owner, system_byte);

        let mut transaction = Secs1MessageTransaction::new_recv(key);
        transaction.handle_receive(recv_blocks.into_iter().next().unwrap());

        let effects = drain_effects(&mut transaction);
        assert!(
            effects.contains(&Secs1TransactionEffect::StartTimeout(SecsTimeoutUnit::T4(
                key
            )))
        );
        assert!(!effects.contains(&Secs1TransactionEffect::TransactionEnd));

        transaction
            .handle_timeout(SecsTimeoutUnit::T4(key))
            .unwrap();

        let effects = drain_effects(&mut transaction);
        assert!(effects.contains(&Secs1TransactionEffect::ErrorOccured(
            SecsTransportError::Timeout(SecsTimeoutUnit::T4(key))
        )));
        assert!(effects.contains(&Secs1TransactionEffect::TransactionEnd));
        assert!(matches!(transaction.state, Secs1TransactionState::End));
    }

    /// primary recv 후 secondary를 보내야 하는 경우
    #[test]
    fn test_recv_primary_need_reply() {
        let device_id = DeviceId(10);
        let system_byte = SystemByte(101);

        // Remote -> Local primary
        let msg = build_primary_message(device_id, system_byte, Rbit::REVERSE, true);
        let stream = msg.header.stream;
        let function = msg.header.function;

        let recv_blocks = encode(msg).unwrap();

        // 내가 응답해야 하는 케이스
        let owner = TransactionOwner::Remote;
        let key = TransactionKey::new(owner, system_byte);

        let mut transaction = Secs1MessageTransaction::new_recv(key);

        // primary block 수신
        transaction.handle_receive(recv_blocks.into_iter().next().unwrap());

        let effects = drain_effects(&mut transaction);

        assert!(effects.contains(&Secs1TransactionEffect::RecvComplete));
        assert!(effects.contains(&Secs1TransactionEffect::ReplyRequired(key,)));
        // 트랜잭션 종료 판정 X. 대기 필요
        assert!(!effects.contains(&Secs1TransactionEffect::TransactionEnd));

        assert!(matches!(transaction.state, Secs1TransactionState::WaitSend));

        // reply 생성
        let reply_msg = build_secondary_message(device_id, system_byte, Rbit::FORWARD, false);
        transaction.handle_reply(reply_msg);

        assert!(matches!(
            transaction.state,
            Secs1TransactionState::Send { .. }
        ));

        // 첫 block 송신
        let block = transaction.poll_write().unwrap();

        let signal = Secs1MessageSignal::BlockSendSuccess {
            header: block.header,
        };

        transaction.handle_signal(signal);

        let effects = drain_effects(&mut transaction);

        assert!(effects.contains(&Secs1TransactionEffect::SendComplete));
        assert!(effects.contains(&Secs1TransactionEffect::TransactionEnd));

        assert!(matches!(transaction.state, Secs1TransactionState::End));
    }

    // 상대방이 블록 던지고, 응답은 필요 없는 경우
    #[test]
    fn test_recv_primary_no_reply() {
        let device_id = DeviceId(10);
        let system_byte = SystemByte(101);

        // Remote -> Local primary
        let msg = build_primary_message(device_id, system_byte, Rbit::REVERSE, false);

        let recv_blocks = encode(msg).unwrap();

        // 상대방이 트랜잭션 주도
        let owner = TransactionOwner::Remote;
        let key = TransactionKey::new(owner, system_byte);

        let mut transaction = Secs1MessageTransaction::new_recv(key);

        // primary block 수신
        transaction.handle_receive(recv_blocks.into_iter().next().unwrap());

        let effects = drain_effects(&mut transaction);

        // 수신 성공
        assert!(effects.contains(&Secs1TransactionEffect::RecvComplete));
        assert!(effects.contains(&Secs1TransactionEffect::TransactionEnd));
        // reply 모드 전이 X
        assert!(!effects
            .iter()
            .any(|effect| { matches!(effect, Secs1TransactionEffect::ReplyRequired(..)) }));

        assert!(matches!(transaction.state, Secs1TransactionState::End));
    }
}

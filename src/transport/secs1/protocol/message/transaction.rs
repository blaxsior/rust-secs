use crate::transport::error::SecsTransportError;
use crate::transport::secs1::block::Secs1Block;
use crate::transport::secs1::{block::Secs1BlockHeader, protocol::message::Secs1MessageSignal};
use crate::transport::{SecsTimeoutUnit, TransactionKey};

use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::mem;
use log;
use secs_ii::{FunctionId, StreamId};

#[derive(Debug)]
pub enum Secs1TransactionState {
    /// 트랜잭션 생성 초기 대기 상태
    Idle,

    /// 메시지를 전송 중인 상태
    Send {
        /// 전송해야 할 블록 목록(남은 블록)
        blocks: VecDeque<Secs1Block>,
    },

    /// Primary 전송 후 Secondary Message 대기 중 (T3: reply Timer)
    WaitRecv,

    /// Primary 수신 후 Secondary Message 전송 대기 중
    WaitSend(TransactionKey),

    /// Message 수신 중인 상태 (T4: inter block timer)
    Recv {
        /// 수신 중인 블록 목록
        blocks: Vec<Secs1Block>,
    },
    /// 트랜잭션 종료
    End,
}

// state 구현은 transaction에서만 접근 가능
impl Secs1TransactionState {
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

    fn is_recv_end(&self) -> bool {
        match self {
            Self::Recv { blocks } => blocks.last().is_some_and(|b| b.header.is_end()),
            _ => false,
        }
    }
}

#[derive(PartialEq, Eq)]
// 트랜잭션에 의한 처리가 필요한 경우
pub enum Secs1TransactionEffect {
    /// Timeout 실행
    StartTimeout(SecsTimeoutUnit),
    /// Timeout 초기화
    ClearTimeout(SecsTimeoutUnit),
    /// 트랜잭션 중 에러 발생
    ErrorOccured(SecsTransportError),
    /// 메시지 수신 성공 -> transaction 삭제 필요
    RecvComplete(Vec<Secs1Block>),

    /// 메시지에 대해 응답이 필요 -> TransactionKey
    ReplyRequired(StreamId, FunctionId, TransactionKey),
    /// 메시지 송신 성공
    SendComplete,
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
    effects: VecDeque<Secs1TransactionEffect>,
    outgoing_blocks: VecDeque<Secs1Block>,
}

impl Secs1MessageTransaction {
    fn new(id: TransactionKey) -> Self {
        Self {
            id,
            state: Secs1TransactionState::Idle,
            last_header: None,
            expected_header: None,
            effects: VecDeque::new(),
            outgoing_blocks: VecDeque::new(),
        }
    }

    pub fn new_send(id: TransactionKey, blocks: Vec<Secs1Block>) -> Self {
        let mut tx = Self::new(id);
        tx.enter_send(blocks);

        tx
    }

    pub fn new_recv(id: TransactionKey) -> Self {
        let mut tx = Self::new(id);
        tx.enter_recv();

        tx
    }

    pub fn handle_receive(&mut self, block: Secs1Block) {
        // 중복 block 발생 -> discard 후 recv 대기
        if self.is_duplicate(&block.header) {
            return;
        }
        // 정상적인 블록 -> dup check를 위한 헤더 저장
        self.last_header = Some(block.header);

        // 기대한 블록이 맞는지 검사
        if !self.is_expected(&block.header) {
            // primary first 아님 -> Block Error 발생
            if !block.header.is_primary() || !block.header.is_first_block() {
                self.emit_effect(Secs1TransactionEffect::ErrorOccured(
                    SecsTransportError::UnexpectedBlock(block.header),
                ));
                return;
            }
        }

        if let Secs1TransactionState::WaitRecv = self.state {
            self.enter_recv();
        }

        self.state.recv_next(block);

        if let Secs1TransactionState::Recv { blocks } = &self.state {
            let block = match blocks.last() {
                Some(b) => b,
                None => return,
            };

            let header = block.header;

            // timer 초기화
            if header.is_first_block() {
                // secondary first: cancel reply timer -> wait recv 상태
                if header.is_secondary() {
                    self.cancel_reply_timer();
                }
            } else {
                // not first: cancel inter block timer
                self.cancel_inter_block_timer();
            }

            if header.is_end() {
                // recv 성공, 블록 상위 전송
                let recv = mem::replace(&mut self.state, Secs1TransactionState::End);
                let Secs1TransactionState::Recv { blocks } = recv else {
                    unreachable!();
                };

                // primary message + secondary message 대기 케이스
                if header.is_primary() && header.need_reply() {
                    // 전송 대기 상태로 전이 (handle_send 시 transaction 매칭 로직 필요)
                    self.enter_wait_send(&header);

                    self.emit_effect(Secs1TransactionEffect::ReplyRequired(
                        header.stream,
                        header.function.reply(),
                        self.id,
                    ));
                }

                self.emit_effect(Secs1TransactionEffect::RecvComplete(blocks));
                self.enter_end();
            } else {
                self.set_inter_block_timer(&header);
            }
        };
    }

    pub fn handle_signal(&mut self, signal: Secs1MessageSignal) {
        match signal {
            Secs1MessageSignal::BlockSendSuccess { header } => {
                self.on_send(&header);
            }
            Secs1MessageSignal::BlockSendFailed { .. } => {
                // 전송 실패 알림
                self.emit_effect(Secs1TransactionEffect::ErrorOccured(
                    SecsTransportError::SendFailed(self.id),
                ));
                // 트랜잭션 종료
                self.enter_end();
            }
        }
    }

    pub fn handle_reply(&mut self, key: &TransactionKey, blocks: Vec<Secs1Block>) {
        if let Secs1TransactionState::WaitSend(stored_key) = self.state {
            // 내부 저장한 키와 비교해서 트랜잭션 키가 다르면 전송 실패로 처리
            if stored_key != *key {
                log::error!(
                    "transaction key different. expected {:?} found {:?}",
                    stored_key,
                    key
                );
                self.emit_effect(Secs1TransactionEffect::ErrorOccured(
                    SecsTransportError::SendFailed(*key),
                ));
                return;
            }

            // send 모드 진입
            self.enter_send(blocks);
        }
    }

    fn send_next(&mut self) {
        let result = self.state.send_next();

        if let Some(block) = result {
            self.last_header = Some(block.header);
            self.outgoing_blocks.push_back(block);
        }
    }

    pub fn poll_send(&mut self) -> Option<Secs1Block> {
        self.outgoing_blocks.pop_front()
    }

    fn switch_state(&mut self, state: Secs1TransactionState) {
        log::debug!("state changed from {:?} to {:?}", self.state, state);
        self.state = state;
    }

    /// end state로 진입한다.
    fn enter_end(&mut self) {
        self.switch_state(Secs1TransactionState::End);
        self.emit_effect(Secs1TransactionEffect::TransactionEnd);
        log::debug!("transaction end");
    }

    /// wait send state로 진입한다.
    fn enter_wait_send(&mut self, header: &Secs1BlockHeader) {
        self.switch_state(Secs1TransactionState::WaitSend(self.id));

        self.emit_effect(Secs1TransactionEffect::ReplyRequired(
            header.stream,
            header.function.reply(),
            self.id,
        ));
    }

    /// wait recv state로 진입한다.
    fn enter_wait_recv(&mut self, header: &Secs1BlockHeader) {
        self.switch_state(Secs1TransactionState::WaitRecv);
        // reply timer 시작
        self.set_reply_timer(header);
        log::debug!("switch to wait recv");
    }

    /// send state로 진입한다.
    fn enter_send(&mut self, blocks: Vec<Secs1Block>) {
        self.switch_state(Secs1TransactionState::new_send(blocks));
        self.send_next();
    }

    /// recv state로 진입한다.
    fn enter_recv(&mut self) {
        self.switch_state(Secs1TransactionState::new_recv());
    }

    /// send가 발생했을 때 동작을 정의
    fn on_send(&mut self, header: &Secs1BlockHeader) {
        if let Secs1TransactionState::Send { blocks } = &self.state {
            // 블록이 비어 있는 경우
            if blocks.is_empty() {
                log::debug!("message send success. owner = {:?}", self.id.owner);
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
    }

    /// 외부로 transaction에 의한 effect 알림
    fn emit_effect(&mut self, effect: Secs1TransactionEffect) {
        self.effects.push_back(effect);
    }

    // /// 외부에서 트랜잭션 effect 수신
    // pub fn poll_effect(&mut self) -> Option<Secs1TransactionEffect> {
    //     return self.effects.pop_front();
    // }

    /// 외부에서 트랜잭션 effect 수신
    pub fn poll_effects(&mut self) -> Vec<Secs1TransactionEffect> {
        Vec::from(mem::take(&mut self.effects))
    }

    /// 기대한 블록이 맞는지 체크
    fn is_expected(&self, actual: &Secs1BlockHeader) -> bool {
        if let Some(expected) = &self.expected_header {
            match self.state {
                Secs1TransactionState::Recv { .. } => {
                    Self::is_expected_header_for_inter_block(expected, actual)
                }
                Secs1TransactionState::WaitRecv => {
                    Self::is_expected_header_for_reply(expected, actual)
                }
                _ => false,
            }
        } else {
            false
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
        self.emit_effect(Secs1TransactionEffect::ClearTimeout(SecsTimeoutUnit::T4(
            self.id,
        )));
    }

    /// reply timer(T3) 지정 + timer 체크 조건 설정
    fn set_reply_timer(&mut self, header: &Secs1BlockHeader) {
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
        self.emit_effect(Secs1TransactionEffect::ClearTimeout(SecsTimeoutUnit::T3(
            self.id,
        )));
    }

    // /// 헤더가 이전에 수신한 헤더와 동일한지 체크 (중복 수신 방지)
    fn is_duplicate(&self, header: &Secs1BlockHeader) -> bool {
        if let Some(last) = &self.last_header {
            return last == header;
        }
        return false;
    }

    pub fn get_last_header(&self) -> Option<Secs1BlockHeader> {
        self.last_header
    }
}

#[cfg(test)]
mod tests {
    // use secs_ii::{FunctionId, StreamId, item::Secs2Variant};

    use std::collections::VecDeque;

    use secs_ii::{FunctionId, Secs2Message, StreamId, item::Secs2Variant};

    use crate::{
        core::SecsMessage,
        transport::{
            DeviceId, Rbit, SecsTimeoutUnit, SystemByte, TransactionKey, TransactionOwner,
            secs1::{
                convert::encode,
                protocol::message::{
                    Secs1MessageSignal,
                    transaction::{
                        Secs1MessageTransaction, Secs1TransactionEffect, Secs1TransactionState,
                    },
                },
            },
        },
    };

    fn build_primary_message(
        device_id: DeviceId,
        system_byte: SystemByte,
        rbit: Rbit,
        need_reply: bool,
    ) -> SecsMessage {
        let stream = StreamId(1);
        let function = FunctionId(3);
        let body = Secs2Variant::list(vec![Secs2Variant::uint4(3001), Secs2Variant::uint4(3002)]);
        let payload = Secs2Message::new(stream, function, need_reply, body);
        let msg = SecsMessage::new(device_id, system_byte, rbit, payload);

        msg
    }

    fn build_secondary_message(
        device_id: DeviceId,
        system_byte: SystemByte,
        rbit: Rbit,
        need_reply: bool,
    ) -> SecsMessage {
        let stream = StreamId(1);
        let function = FunctionId(4);
        let body = Secs2Variant::list(vec![Secs2Variant::uint8(1500), Secs2Variant::uint8(1501)]);
        let payload = Secs2Message::new(stream, function, need_reply, body);
        let msg = SecsMessage::new(device_id, system_byte, rbit, payload);

        msg
    }

    /// primary send 후 secondary를 요구하는 경우
    #[test]
    fn test_active_send_primary_need_reply() {
        let device_id = DeviceId(10);
        let system_byte = SystemByte(101);

        let msg = build_primary_message(device_id, system_byte, Rbit::FORWARD, true);

        let send_blocks = encode(msg).unwrap();
        // 내가 주인인 케이스
        let owner = TransactionOwner::Local;
        let key = TransactionKey::new(owner, system_byte);

        let mut transaction = Secs1MessageTransaction::new_send(key, send_blocks);

        let block = transaction.poll_send().unwrap();

        let signal = Secs1MessageSignal::BlockSendSuccess {
            header: block.header,
        };
        transaction.handle_signal(signal);

        // send 후 recv 대기 시 reply timer 활성화
        let effects = transaction.poll_effects();
        assert!(
            effects.contains(&Secs1TransactionEffect::StartTimeout(SecsTimeoutUnit::T3(
                key
            )))
        );

        assert!(matches!(transaction.state, Secs1TransactionState::WaitRecv));

        let msg = build_secondary_message(device_id, system_byte, Rbit::REVERSE, false);
        let mut recv_blocks = VecDeque::from(encode(msg).unwrap());

        transaction.handle_receive(recv_blocks.pop_front().unwrap());

        let effects = transaction.poll_effects();
        assert!(effects.contains(&Secs1TransactionEffect::TransactionEnd));

        assert!(matches!(transaction.state, Secs1TransactionState::End));
    }

    #[test]
    fn test_send() {}
}

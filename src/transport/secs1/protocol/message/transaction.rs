use crate::transport::error::SecsTransportError;
use crate::transport::secs1::block::Secs1Block;
use crate::transport::secs1::{block::Secs1BlockHeader, protocol::message::Secs1MessageSignal};
use crate::transport::{SecsTimeoutUnit, TransactionKey, TransactionOwner};

use alloc::collections::VecDeque;
use alloc::vec;
use alloc::vec::Vec;
use core::mem;
use log;
use secs_ii::{FunctionId, StreamId};
#[derive(Debug)]
pub enum Secs1TransactionState {
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

impl Secs1TransactionState {
    /// recv 모드로 진입한다.
    pub fn switch_to_receive(&mut self, block: Secs1Block) {
        *self = Secs1TransactionState::Recv {
            blocks: vec![block],
        };
    }

    // send 모드로 진입한다.
    pub fn switch_to_send(&mut self, blocks: Vec<Secs1Block>) {
        *self = Secs1TransactionState::Send {
            blocks: VecDeque::from(blocks),
        };
    }

    pub fn send_next(&mut self) -> Option<Secs1Block> {
        // Send 상태 -> 마지막 블록 전송
        if let Secs1TransactionState::Send { blocks } = self {
            return blocks.pop_front();
        } else {
            log::warn!("cannot send next block. current state: {:?}", self);
            None
        }
    }

    /// recv 모드에서 아이템을 추가한다.
    pub fn recv_next(&mut self, block: Secs1Block) {
        if let Secs1TransactionState::Recv { blocks } = self {
            blocks.push(block);
        } else {
            log::warn!("cannot recv next block. current state: {:?}", self);
        }
    }
}

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
}

impl Secs1MessageTransaction {
    pub fn new(id: TransactionKey, state: Secs1TransactionState) -> Self {
        Self {
            id,
            state,
            last_header: None,
            expected_header: None,
            effects: VecDeque::new(),
        }
    }

    pub fn handle_receive(&mut self, block: Secs1Block) -> Result<(), SecsTransportError> {
        // 중복 block 발생 -> discard 후 recv 대기
        if self.is_duplicate(&block.header) {
            return Ok(());
        }
        // 정상적인 블록 -> dup check를 위한 헤더 저장
        self.last_header = Some(block.header);

        // 기대한 블록이 아닌 경우
        if !self.is_expected(&block.header) {
            // primary first 아님 -> Block Error 발생
            if !block.header.is_primary() || !block.header.is_first_block() {
                return Err(SecsTransportError::UnexpectedBlock(block.header));
            }
        }

        match self.state {
            Secs1TransactionState::Recv { .. } => self.state.recv_next(block),
            Secs1TransactionState::WaitRecv => self.state.switch_to_receive(block),
            _ => panic!("unreachable code"),
        };

        if let Secs1TransactionState::Recv { blocks } = &self.state {
            let block = blocks.last().unwrap();
            let header = block.header;

            {
                // timer 초기화
                if header.is_first_block() {
                    // secondary first: cancel reply timer
                    if header.is_secondary() {
                        self.cancel_reply_timer();
                    }
                } else {
                    // not first: cancel inter block timer
                    self.cancel_inter_block_timer();
                }
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
                    self.state = Secs1TransactionState::WaitSend(self.id);
                    self.emit_effect(Secs1TransactionEffect::ReplyRequired(
                        header.stream,
                        header.function.reply(),
                        self.id,
                    ));
                }

                self.emit_effect(Secs1TransactionEffect::RecvComplete(blocks));
            } else {
                self.set_inter_block_timer(&header);
            }
        };

        Ok(())
    }

    pub fn handle_signal(&mut self, signal: Secs1MessageSignal) -> Option<Secs1Block> {
        match signal {
            Secs1MessageSignal::BlockSendSuccess { header } => {
                self.on_send_progress(&header);
                self.send_next()
            }
            Secs1MessageSignal::BlockSendFailed { .. } => {
                // 전송 실패 알림
                self.emit_effect(Secs1TransactionEffect::ErrorOccured(
                    SecsTransportError::SendFailed(self.id),
                ));
                // 트랜잭션 종료
                self.end_transaction();
                None
            }
        }
    }

    pub fn send_next(&mut self) -> Option<Secs1Block> {
        let result = self.state.send_next();
        if let Some(block) = &result {
            self.last_header = Some(block.header);
        }

        result
    }

    pub fn handle_reply(&mut self, key: &TransactionKey, blocks: Vec<Secs1Block>) {
        if let Secs1TransactionState::WaitSend(stored_key) = self.state {
            // 내부 저장한 키와 비교해서 트랜잭션 키가 다르면 전송 실패로 처리, 트랜잭션 종료
            if stored_key != *key {
                log::error!(
                    "transaction key different. expected {:?} found {:?}",
                    stored_key,
                    key
                );
                self.emit_effect(Secs1TransactionEffect::ErrorOccured(
                    SecsTransportError::SendFailed(*key),
                ));
                // self.end_transaction();
                // timeout에 의한 트랜잭션 종료로 처리하기 위해 대기
                return;
            }

            self.switch_to_send(blocks);
        }
    }

    pub fn switch_to_send(&mut self, blocks: Vec<Secs1Block>) {
        self.state = Secs1TransactionState::Send {
            blocks: VecDeque::from(blocks),
        };
        log::debug!("switch state to send");
    }

    fn end_transaction(&mut self) {
        self.state = Secs1TransactionState::End;
        self.emit_effect(Secs1TransactionEffect::TransactionEnd);
        log::debug!("transaction end");
    }

    fn switch_to_wait_recv(&mut self, header: &Secs1BlockHeader) {
        self.state = Secs1TransactionState::WaitRecv;
        self.set_reply_timer(header);
        log::debug!("switch to wait recv");
    }

    fn on_send_progress(&mut self, header: &Secs1BlockHeader) {
        if let Secs1TransactionState::Send { blocks } = &self.state {
            // 블록이 비어 있는 상태
            if blocks.is_empty() {
                log::debug!("message send success. owner = {:?}", self.id.owner);
                self.emit_effect(Secs1TransactionEffect::SendComplete);

                match self.id.owner {
                    // 내가 primary message block을 다 보낸 상태
                    TransactionOwner::Local => {
                        if header.need_reply() {
                            self.switch_to_wait_recv(&header);
                        } else {
                            self.end_transaction();
                        }
                    }
                    // 상대가 보낸 primary message에 secondary로 응답한 상태
                    // 트랜잭션이 종료됨
                    TransactionOwner::Remote => {
                        self.end_transaction();
                    }
                }
            }
        }
    }

    fn emit_effect(&mut self, effect: Secs1TransactionEffect) {
        self.effects.push_back(effect);
    }

    /// 외부로 메시지를 전달
    pub fn poll_effect(&mut self) -> Option<Secs1TransactionEffect> {
        return self.effects.pop_front();
    }

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

    /// primary + need recv 데이터를 요청받은 경우
    #[test]
    fn test_recv_primary_need_reply() {

    }

    #[test]
    fn test_send() {}
}

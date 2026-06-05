use core::panic;

use crate::{
    transport::{
        SecsTimeoutUnit, TransactionId,
        error::SecsTransportError,
        secs1::{
            block::{Secs1Block, Secs1BlockHeader},
            config::{DeviceId, Secs1TransportConfig},
        },
    },
    util::time::{TimeoutId, TimeoutManager, TimeoutTicket},
};

use alloc::collections::BTreeMap;
use alloc::collections::VecDeque;
use alloc::vec;
use alloc::vec::Vec;
use sansio::Protocol;

use secs_ii::{SecsMessage, item::Secs2Variant};

/// secs-i message protocol 수행 중 외부에서 주입하는 이벤트(블록 정상 송신 등)
pub enum Secs1MessageSignal {
    /// 대상 블록 전송에 성공함
    BlockSendSuccess { header: Secs1BlockHeader },
    /// 대상 블록 전송을 실패
    BlockSendFailed { header: Secs1BlockHeader },
}

/// secs-i message protocol 수행 중 발생하는 이벤트
pub enum Secs1MessageEvent {
    /// 메시지를 성공적으로 송신
    MessageSent { transaction_id: TransactionId },
    /// 메시지를 성공적으로 수신
    MessageReceived { transaction_id: TransactionId },

    /// 송신 대기 중인 메시지가 타임아웃 발생
    MessageSendTimeout {
        transaction_id: TransactionId,
        timeout_unit: SecsTimeoutUnit,
    },
    /// 수신 대기 중인 메시지가 타임아웃 발생
    MessageReceiveTimeout {
        transaction_id: TransactionId,
        timeout_unit: SecsTimeoutUnit,
    },
    /// 비정상적인 상태 전이 등 에러가 발생한 경우
    ErrorOccured { error: SecsTransportError },
}
// 블록 송/수신 실패 케이스의 경우 Block Transfer 수준에서 timeout으로 표현되므로 문제 X

/// 트랜잭션 상 역할
pub enum TransactionRole {
    /// PRIMARY 보냄: SEND -> WAIT_REPLY -> RESPONSE(if needed)
    Initiator,
    /// PRIMARY 응답: RECV -> SEND case
    Responder,
}

pub enum Secs1MessageTransactionState {
    /// 트랜잭션 생성 초기 상태 (구체적인 전이 전 상태)
    Idle,
    /// 메시지를 전송 중인 상태
    Sending {
        /// 전송해야 할 블록 목록
        blocks: Vec<Secs1Block>,
        /// 전송할 block의 인덱스 (0부터 시작)
        vec_idx: usize,
    },

    /// Primary 전송 후 Secondary Message 대기 중 (T3: reply Timer)
    WaitReply { expected_header: Secs1BlockHeader },

    /// Message 수신 중인 상태 (T4: inter block timer)
    Recv {
        expected_header: Secs1BlockHeader,
        blocks: Vec<Secs1Block>,
    },
}

impl Secs1MessageTransactionState {
    /// recv 모드로 진입한다.
    pub fn switch_to_receive(&mut self, block: Secs1Block) {
        *self = Secs1MessageTransactionState::Recv {
            expected_header: block.header,
            blocks: vec![block],
        };
    }

    // send 모드로 진입한다.
    pub fn switch_to_send(&mut self, msg: Secs2Variant) {}

    /// fn recv_item()

    /// recv 모드에서 아이템을 추가한다.
    pub fn recv_next(&mut self, block: Secs1Block) {
        if let Secs1MessageTransactionState::Recv {
            expected_header,
            blocks,
        } = self
        {
            *expected_header = block.header;
            blocks.push(block);
        }
    }

    // pub fn is_complete(&self) -> bool {
    //     match self {
    //         Secs1MessageTransactionState::Receiving {
    //             expected_header, ..
    //         } => expected_header.is_end(),
    //         _ => false,
    //     }
    // }
}

// 트랜잭션에 의한 처리가 필요한 경우
pub enum Secs1TransactionEffect {
    StartTimeout(SecsTimeoutUnit),
    ClearTimeout(SecsTimeoutUnit),
    /// 트랜잭션이 중단됨
    TransactionAborted,
}

/// 트랜잭션을 표현하는 객체
pub struct Secs1MessageTransaction {
    /// 트랜잭션에 부여된 system byte 값
    id: TransactionId,
    /// 트랜잭션 역할
    role: TransactionRole,
    /// 트랜잭션 진행 상태
    state: Secs1MessageTransactionState,
    last_header: Option<Secs1BlockHeader>,
}

impl Secs1MessageTransaction {
    pub fn new(id: TransactionId, role: TransactionRole) -> Self {
        Self {
            id,
            role,
            state: Secs1MessageTransactionState::Idle,
            last_header: None,
        }
    }

    pub fn handle_receive(&mut self, block: Secs1Block) -> Result<(), SecsTransportError> {
        // 중복 block 발생 -> discard
        if self.is_duplicate(&block.header) {
            return Ok(());
        }
        // 정상적인 블록 -> dup check를 위한 헤더 저장
        self.last_header = Some(block.header);

        if !self.is_expected(&block.header) {
            // 에러 발생!
            return Err(SecsTransportError::UnexpectedBlock(block.header));
        }

        match self.state {
            Secs1MessageTransactionState::WaitReply { .. } => self.state.switch_to_receive(block),
            Secs1MessageTransactionState::Recv { .. } => self.state.recv_next(block),
            _ => panic!("unreachable code"),
        };
        Ok(())
    }

    /// 기대한 블록이 맞는지 체크
    fn is_expected(&self, header: &Secs1BlockHeader) -> bool {
        match &self.state {
            Secs1MessageTransactionState::WaitReply { expected_header } => {
                expected_header == header
            }
            Secs1MessageTransactionState::Recv {
                expected_header, ..
            } => expected_header == header,
            _ => panic!("invalid state"),
        }
    }

    // /// 헤더가 이전에 수신한 헤더와 동일한지 체크 (중복 수신 방지)
    fn is_duplicate(&self, header: &Secs1BlockHeader) -> bool {
        if let Some(last) = &self.last_header {
            return last == header;
        }
        return false;
    }

    // /// 헤더가 현재 기대하는 헤더인지 체크
    // fn is_expected(&self, header: &Secs1BlockHeader) -> bool {
    //     match &self.state {
    //         Secs1MessageTransactionState::Receiving {
    //             expected_header, ..
    //         } => expected_header == header,
    //         Secs1MessageTransactionState::WaitingForReply { expected_header } => {
    //             expected_header == header
    //         }
    //         _ => false,
    //     }
    // }

    // fn is_primary_first_block(&self, header: &Secs1BlockHeader) -> bool {
    //     header.is_primary() && header.is_first_block()
    // }

    // fn reset_to_idle(&mut self) {
    //     self.timeout_manager.cancel(SecsTimeoutUnit::T3);
    //     self.state = Secs1MessageTransactionState::Idle;
    // }
}

pub struct Secs1MessageMachine {
    /** 상태 임시 저장 */

    /// block 수신 된 데이터를 임시 보관하는 큐. handle_read
    incoming_blocks: VecDeque<Secs1Block>,
    /// block 송신할 데이터를 임시로 보관하는 큐. poll_write
    outgoing_blocks: VecDeque<Secs1Block>,

    /// secs-ii msg 송신 요청한 블록들을 임시로 저장하는 큐. handle_write
    incoming_msgs: VecDeque<SecsMessage>,
    /// secs-ii msg 수신한 블록들을 임시로 저장하는 큐. poll_read
    outgoing_msgs: VecDeque<SecsMessage>,

    /// 외부에서 발생을 알리는 타임아웃 정보
    incoming_timeouts: VecDeque<TimeoutTicket>,
    /// 외부로 타임아웃 시작 알리는 큐. poll_timeout
    outgoing_timeouts: VecDeque<TimeoutTicket>,

    /// 외부에서 전달하는 신호. push_event
    incoming_signals: VecDeque<Secs1MessageSignal>,
    /// 외부 발송할 이벤트 목록을 저장해두는 큐. poll_event
    outgoing_events: VecDeque<Secs1MessageEvent>,
    /// timeout 처리 객체
    timeout_manager: TimeoutManager,
    /// 통신 중인 디바이스의 ID
    device_id: DeviceId,
    /// 다음에 사용할 트랜잭션 ID
    next_transaction_id: TransactionId,
    transaction_map: BTreeMap<TransactionId, Secs1MessageTransaction>,
}

impl Secs1MessageMachine {
    pub fn new(config: &Secs1TransportConfig) -> Self {
        Self {
            incoming_blocks: VecDeque::new(),
            outgoing_blocks: VecDeque::new(),
            incoming_msgs: VecDeque::new(),
            outgoing_msgs: VecDeque::new(),
            incoming_timeouts: VecDeque::new(),
            outgoing_timeouts: VecDeque::new(),
            incoming_signals: VecDeque::new(),
            outgoing_events: VecDeque::new(),
            timeout_manager: TimeoutManager::new(),
            device_id: config.device_id,
            next_transaction_id: TransactionId(1),
            transaction_map: BTreeMap::new(),
        }
    }
    /// 외부 시스템에 전송할 메시지를 담는다.
    fn emit_event(&mut self, output: Secs1MessageEvent) {
        self.outgoing_events.push_back(output);
    }

    /// timeout을 시작한다.
    fn start_timeout(&mut self, timeout: SecsTimeoutUnit) {
        let ticket = self.timeout_manager.issue(timeout);
        self.outgoing_timeouts.push_back(ticket);
    }

    /// timeout을 취소한다.
    fn cancel_timeout(&mut self, timeout: SecsTimeoutUnit) {
        self.timeout_manager.cancel(timeout);
    }

    pub fn generate_transaction_id(&mut self) -> TransactionId {
        let current = self.next_transaction_id;
        self.next_transaction_id = self.next_transaction_id.next();
        return current;
    }

    pub fn run(&mut self) {
        // 1. 전달된 timeout 처리
        self.process_timeout();
        // 2. 전달된 이벤트 처리(timeout과 유사)
        self.process_signal();
        // 1. receive 상태 처리
        self.process_receive();
        // 2. signal 처리 및 signal 상태 전이
        self.process_send();
    }

    fn process_receive(&mut self) {
        while let Some(block) = self.incoming_blocks.pop_front() {
            // Routing ERROR: 내가 다루는 deviceId가 아님 -> 에러 알리고 무시
            let transaction_id = block.header.system_bytes;

            if !self.is_known_device(&block.header.device_id) {
                self.handle_unknown_device(block);
                continue;
            }

            let mut transaction = match self.find_transaction(&transaction_id) {
                Some(tx) => tx,
                None => {
                    // 적절한 예외처리 후 continue 진행.
                    // 존재하지 않는 transaction_id가 도착하는 경우는 거의 없음
                    continue;
                }
            };

            transaction.handle_receive(block);
        }
    }

    /// receive 중 unknown device 발견 시 대응 처리
    fn handle_unknown_device(&mut self, block: Secs1Block) {
        // 상위 측으로 이벤트 알림
        self.emit_event(Secs1MessageEvent::ErrorOccured {
            error: SecsTransportError::UnknownDeviceId(block.header.device_id),
        });
        // 트랜잭션으로 등록되어 있었던 경우라면 트랜잭션도 취소 처리
        self.remove_transaction(&block.header.system_bytes);
    }

    fn process_send(&mut self) {}

    fn process_timeout(&mut self) {}

    // 외부 signal에 대해 아이템을 처리한다.
    fn process_signal(&mut self) {}

    /// 대상 트랜잭션을 찾아 반환한다.
    fn find_transaction(&mut self, id: &TransactionId) -> Option<&mut Secs1MessageTransaction> {
        return self.transaction_map.get_mut(id);
    }

    /// 대상 트랜잭션을 제거한다.
    fn remove_transaction(&mut self, id: &TransactionId) {
        self.transaction_map.remove(id);
    }

    /// device id가 알려진 장치인지 체크
    fn is_known_device(&self, device_id: &DeviceId) -> bool {
        self.device_id == *device_id
    }
}

impl Protocol<Secs1Block, SecsMessage, Secs1MessageSignal> for Secs1MessageMachine {
    type Rout = SecsMessage;
    type Wout = Secs1Block;
    type Eout = Secs1MessageEvent;
    type Error = SecsTransportError;
    type Time = TimeoutTicket;

    fn handle_read(&mut self, msg: Secs1Block) -> Result<(), Self::Error> {
        Ok(())
    }

    fn poll_read(&mut self) -> Option<Self::Rout> {
        self.outgoing_msgs.pop_front()
    }

    fn handle_write(&mut self, msg: SecsMessage) -> Result<(), Self::Error> {
        self.incoming_msgs.push_back(msg);
        Ok(())
    }

    fn poll_write(&mut self) -> Option<Self::Wout> {
        self.outgoing_blocks.pop_front()
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

        // 2. 상태(State)와 유닛(Unit)을 튜플로 묶어서 매칭
        // match (&self.state, expired_unit) {
        //     // LINECONTROL or SEND 상태 & T2 타임아웃
        //     (Secs1BlockTransferState::LINECONTROL, SecsTimeoutUnit::T2)
        //     | (Secs1BlockTransferState::SEND(_), SecsTimeoutUnit::T2) => {
        //         self.retrigger_line_control()
        //     }

        //     // RECEIVE 상태 & T2 타임아웃 (WaitingLength)
        //     (
        //         Secs1BlockTransferState::RECEIVE(ReceiveState::WaitingLength),
        //         SecsTimeoutUnit::T2,
        //     ) => {
        //         self.emit_event(Secs1BlockTransferEvent::ReceiveFailed {
        //             error: SecsTransportError::Timeout(SecsTimeoutUnit::T2),
        //         });
        //         self.completion_with_send_nak(SecsTimeoutUnit::T2);
        //     }
        //     // RECEIVE 상태 & T1 타임아웃 (WaitingData or InvalidBlock)
        //     (
        //         Secs1BlockTransferState::RECEIVE(ReceiveState::WaitingData { .. }),
        //         SecsTimeoutUnit::T1,
        //     )
        //     | (Secs1BlockTransferState::RECEIVE(ReceiveState::InvalidBlock), SecsTimeoutUnit::T1) =>
        //     {
        //         self.completion_with_send_nak(SecsTimeoutUnit::T1);
        //     }
        //     // 상태와 타임아웃 유닛이 매칭되지 않으면 무시
        //     _ => {}
        // }

        Ok(())
    }

    fn poll_timeout(&mut self) -> Option<Self::Time> {
        self.outgoing_timeouts.pop_front()
    }

    fn handle_event(&mut self, signal: Secs1MessageSignal) -> Result<(), Self::Error> {
        self.incoming_signals.push_back(signal);
        Ok(())
    }

    /// 외부 시스템에서 전송할 메시지를 가져간다.
    fn poll_event(&mut self) -> Option<Self::Eout> {
        self.outgoing_events.pop_front()
    }
}

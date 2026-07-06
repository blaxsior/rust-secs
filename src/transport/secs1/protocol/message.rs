use core::mem;
use core::panic;

use crate::transport::secs1::convert::decode;
use crate::{
    transport::{
        ConnectionRole, SecsTimeoutUnit, SystemByte, TransactionId,
        error::SecsTransportError,
        secs1::{
            block::{Secs1Block, Secs1BlockHeader},
            config::{Secs1TransportConfig},
        },
    },
    util::time::{TimeoutManager, TimeoutTicket},
};

use alloc::collections::BTreeMap;
use alloc::collections::VecDeque;
use alloc::vec;
use alloc::vec::Vec;
use sansio::Protocol;

use secs_ii::DeviceId;
use secs_ii::{FunctionId, SecsMessage};

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

    /// 메시지 송수신 중 타임아웃 발생
    MessageTimeout {
        transaction_id: TransactionId,
        timeout_unit: SecsTimeoutUnit,
    },
    /// 비정상적인 상태 전이 등 에러가 발생한 경우
    ErrorOccured { error: SecsTransportError },
}

pub enum Secs1TransactionState {
    /// 메시지를 전송 중인 상태
    Send {
        /// 전송해야 할 블록 목록(남은 블록)
        blocks: VecDeque<Secs1Block>,
    },

    /// Primary 전송 후 Secondary Message 대기 중 (T3: reply Timer)
    WaitRecv,

    /// Primary 수신 후 Secondary Message 전송 대기 중
    WaitSend(TransactionId),

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
        }

        // Send 상태가 아니거나, 보낼 블록이 더 이상 없는 경우
        None
    }

    /// recv 모드에서 아이템을 추가한다.
    pub fn recv_next(&mut self, block: Secs1Block) {
        if let Secs1TransactionState::Recv { blocks } = self {
            blocks.push(block);
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
    /// 메시지 송신 성공
    SendComplete,
}

/// 트랜잭션을 표현하는 객체
pub struct Secs1MessageTransaction {
    /// 트랜잭션을 구분하는 ID 값
    id: TransactionId,
    /// 트랜잭션 진행 상태
    state: Secs1TransactionState,
    last_header: Option<Secs1BlockHeader>,
    expected_header: Option<Secs1BlockHeader>,
    effects: VecDeque<Secs1TransactionEffect>,
}

impl Secs1MessageTransaction {
    pub fn new(id: TransactionId, state: Secs1TransactionState) -> Self {
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
                return Err(SecsTransportError::BlockError(block.header));
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
                }

                self.emit_effect(Secs1TransactionEffect::RecvComplete(blocks));
            } else {
                self.set_inter_block_timer(&header);
            }
        };

        Ok(())
    }

    pub fn poll_send(&mut self) -> Option<Secs1Block> {
        self.state.send_next()
    }

    fn emit_effect(&mut self, effect: Secs1TransactionEffect) {
        self.effects.push_back(effect);
    }

    /// 외부로 메시지를 전달
    pub fn poll_effect(&mut self) -> Option<Secs1TransactionEffect> {
        return self.effects.pop_front();
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
            && expected.system_bytes == actual.system_bytes;
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
            && expected.system_bytes == actual.system_bytes;
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
            rbit: !header.rbit,
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
    /// 통신 중인 디바이스의 세션 ID
    // source_id: SourceId,
    /// 연결 모드 -> active or passive
    role: ConnectionRole,

    /// 다음에 사용할 트랜잭션 ID (source 주도 기준)
    next_system_byte: SystemByte,
    transaction_map: BTreeMap<TransactionId, Secs1MessageTransaction>,
}

pub enum Secs1MessageDirection {
    // 주는 중
    Send,
    // 받는 중
    Recv,
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
            // source_id: counterpart,
            role: config.local_role,
            next_system_byte: SystemByte(0),
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

    /// transaction_id를 만든다
    // fn generate_transaction_id(&mut self) -> TransactionId {
    //     let current = self.next_system_byte;
    //     self.next_system_byte = current.next();
    //     return TransactionId(true, current);
    // }

    /// header로부터 transaction_id를 획득
    fn get_transaction_id(&self, header: &Secs1BlockHeader) -> TransactionId {
        let sender_is_self = match self.role {
            ConnectionRole::Active => header.is_from_active(),
            ConnectionRole::Passive => header.is_from_passive(),
        };


        TransactionId(sender_is_self, header.system_bytes)
    }

    pub fn run(&mut self) {
        // 1. 전달된 timeout 처리
        self.process_timeout();
        // 2. 전달된 이벤트 처리 -> 블록 전송 상태 체크 -> send 재개 작업
        self.process_signal();
        // 3. receive 상태 처리
        self.process_receive();
    }

    fn process_receive(&mut self) {
        while let Some(block) = self.incoming_blocks.pop_front() {
            // Routing ERROR: 내가 다루는 deviceId가 아님 -> 에러 알리고 무시
            let transaction_id =
                self.get_transaction_id(&block.header);

            if !self.is_known_device(&block.header.device_id) {
                self.handle_unknown_device(block);
                continue;
            }

            let mut effects = Vec::new();
            {
                // 기존 트랜잭션이 있나? 없으면 생성 후 대응
                let transaction = self
                    .transaction_map
                    .entry(transaction_id)
                    .or_insert_with(|| {
                        Secs1MessageTransaction::new(
                            transaction_id,
                            Secs1TransactionState::Recv { blocks: Vec::new() },
                        )
                    });

                let result = transaction.handle_receive(block);

                match result {
                    Ok(_) => {
                        while let Some(effect) = transaction.poll_effect() {
                            effects.push(effect);
                        }
                    }
                    Err(error) => {
                        self.remove_transaction(&transaction_id);
                        self.emit_event(Secs1MessageEvent::ErrorOccured { error });
                    }
                }
            }
            self.handle_effects(effects, &transaction_id);
        }
    }

    fn handle_effects(
        &mut self,
        effects: Vec<Secs1TransactionEffect>,
        transaction_id: &TransactionId,
    ) {
        for effect in effects {
            match effect {
                Secs1TransactionEffect::StartTimeout(secs_timeout_unit) => {
                    self.start_timeout(secs_timeout_unit);
                }
                Secs1TransactionEffect::ClearTimeout(secs_timeout_unit) => {
                    self.cancel_timeout(secs_timeout_unit);
                }
                Secs1TransactionEffect::ErrorOccured(error) => {
                    self.emit_event(Secs1MessageEvent::ErrorOccured { error });
                    self.remove_transaction(transaction_id);
                }
                Secs1TransactionEffect::RecvComplete(blocks) => match decode(blocks) {
                    Ok(msg) => {
                        // 아직 메시지 reply가 필요한 경우
                        if !msg.need_reply {
                            self.remove_transaction(transaction_id);
                        }
                        self.outgoing_msgs.push_back(msg);
                        self.emit_event(Secs1MessageEvent::MessageReceived {
                            transaction_id: *transaction_id,
                        });
                    }
                    Err(error) => {
                        self.emit_event(Secs1MessageEvent::ErrorOccured {
                            error: SecsTransportError::MessageConvertFailed(error),
                        });
                    }
                },
                Secs1TransactionEffect::SendComplete => {
                    self.remove_transaction(transaction_id);
                    self.emit_event(Secs1MessageEvent::MessageSent {
                        transaction_id: *transaction_id,
                    });
                }
            }
        }
    }

    /// receive 중 unknown device 발견 시 대응 처리
    fn handle_unknown_device(&mut self, block: Secs1Block) {
        // 상위 측으로 이벤트 알림
        self.emit_event(Secs1MessageEvent::ErrorOccured {
            error: SecsTransportError::UnknownDeviceId(block.header.device_id),
        });

        let transaction_id =
            self.get_transaction_id(&block.header);
        // 트랜잭션으로 등록되어 있었던 경우라면 트랜잭션도 취소 처리
        self.remove_transaction(&transaction_id);
    }

    fn process_timeout(&mut self) {
        // 1. _now에서 타임아웃 유닛 추출 (구조체에 맞게 메서드 호출)
        while let Some(ticket) = self.incoming_timeouts.pop_front() {
            let is_timeout_valid = self.timeout_manager.fire(&ticket);
            if !is_timeout_valid {
                // 이미 취소된 타임아웃인 경우 -> 무시
                continue;
            }
            let timeout = ticket.timeout;
            let transaction_id = timeout.to_transaction_id();

            // 타임아웃 발생 -> 해당 transaction abort = 제거 처리 후 이벤트 발생
            if self.find_transaction(&transaction_id).is_some() {
                self.remove_transaction(&transaction_id);
                self.emit_event(Secs1MessageEvent::MessageTimeout {
                    transaction_id,
                    timeout_unit: ticket.timeout,
                });
            };

            // timeout 발생 -> 대상 트랜잭션 abort 처리
        }
    }

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
        self.incoming_blocks.push_back(msg);
        self.run();
        Ok(())
    }

    fn poll_read(&mut self) -> Option<Self::Rout> {
        self.outgoing_msgs.pop_front()
    }

    fn handle_write(&mut self, msg: SecsMessage) -> Result<(), Self::Error> {
        self.incoming_msgs.push_back(msg);
        self.run();
        Ok(())
    }

    fn poll_write(&mut self) -> Option<Self::Wout> {
        self.outgoing_blocks.pop_front()
    }

    /// timeout 발생 시 처리
    fn handle_timeout(&mut self, timeticket: Self::Time) -> Result<(), Self::Error> {
        self.incoming_timeouts.push_back(timeticket);
        self.run();
        Ok(())
    }

    fn poll_timeout(&mut self) -> Option<Self::Time> {
        self.outgoing_timeouts.pop_front()
    }

    fn handle_event(&mut self, signal: Secs1MessageSignal) -> Result<(), Self::Error> {
        self.incoming_signals.push_back(signal);
        self.run();
        Ok(())
    }

    /// 외부 시스템에서 전송할 메시지를 가져간다.
    fn poll_event(&mut self) -> Option<Self::Eout> {
        self.outgoing_events.pop_front()
    }
}

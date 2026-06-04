use crate::{
    transport::{
        ConnectionMode,
        error::{SecsTimeoutUnit, SecsTransportError},
        secs1::{
            block::{Secs1Block, Secs1BlockHeader, Secs1HandshakeCode},
            config::Secs1TransportConfig,
        },
    },
    util::time::{TimeoutManager, TimeoutTicket},
};

use alloc::collections::VecDeque;
use alloc::vec::Vec;
use sansio::Protocol;

///
/// SECS-I 통신 block transfer 수행 중 상태
///
enum Secs1LinkState {
    /// 초기 상태
    IDLE,
    /// 전송 방향 수립 중
    LINECONTROL(LineControlState),
    /// 데이터 전송 중
    SEND(SendState),
    /// 데이터 수신 중
    RECEIVE(ReceiveState), // 수신한 length byte
    /// 전송 후 종료 단계. 명세상 존재하나, 각 state에서 바로 IDLE 전이하도록 기능 구현 중
    COMPLETION,
}

enum LineControlState {
    /// ENQ 송신 전
    BeforeSendENQ,
    /// ENQ 송신 후 EOT or ENQ 수신 대기 중
    WaitForResponse,
}

enum ReceiveState {
    /// length byte 수신 대기
    WaitingLength,
    /// length byte 수신 완료 -> data byte 수신 대기
    WaitingData { length: u8, buffer: Vec<u8> },
    /// block을 파싱했는데 데이터가 이상한 경우 -> 남은 T1 기간동안 데이터 전부 버린 후 NAK 전송 + IDLE 복귀
    InvalidBlock,
}

enum SendState {
    /// 아직 아이템을 송신하지 않음
    BeforeSend,
    // 아이템 송신 후 ACK 대기 중
    WaitForAck,
}

/// SECS-I Link에서 발생하는 주요 이벤트
#[derive(Debug, PartialEq, Eq)]
pub enum Secs1LinkEvent {
    /// 블록 송신 성공
    SendSuccess { header: Secs1BlockHeader },
    /// 블록 송신 실패
    SendFailed {
        header: Secs1BlockHeader,
        error: SecsTransportError,
    },
    /// Line control 중 문제 발생
    LineControlFailed,
    /// Line control 단계 재시도
    Retry(u8),

    /// receive 과정에서 실패
    ReceiveFailed { error: SecsTransportError },
    /// 기타 오류
    ErrorOccurred(SecsTransportError),
}

/// SECS-I block transfer을 처리하는 상태 머신
pub struct Secs1LinkMachine {
    /** 상태 임시 저장 */

    /// serial 수신 된 데이터를 임시 보관하는 큐. handle_read
    incoming_buffer: VecDeque<u8>,
    /// serial 송신할 데이터를 임시로 보관하는 큐. poll_write
    outgoing_buffer: VecDeque<u8>,

    /// 송신 요청한 블록들을 임시로 저장하는 큐. handle_write
    incoming_blocks: VecDeque<Secs1Block>,

    /// 수신한 블록들을 임시로 저장하는 큐. poll_read
    outgoing_blocks: VecDeque<Secs1Block>,

    /// 외부로 타임아웃 시작 알리는 큐. poll_timeout
    outgoing_timeout_queue: VecDeque<TimeoutTicket>,
    /// 외부 발송할 이벤트 목록을 저장해두는 큐. poll_event
    event_queue: VecDeque<Secs1LinkEvent>,

    timeout_manager: TimeoutManager,

    /** 내부 상태 값 */
    /// SECS-I block transfer current state
    state: Secs1LinkState,
    /// SECS-I 통신 모드 (ACTIVE/ PASSIVE)
    mode: ConnectionMode,
    // 현재 retry 횟수
    current_retry: u8,
    // 최대 retry 횟수
    max_retry: u8,
}

impl Secs1LinkMachine {
    pub fn new(config: &Secs1TransportConfig) -> Self {
        Self {
            state: Secs1LinkState::IDLE,

            incoming_buffer: VecDeque::new(),
            outgoing_buffer: VecDeque::new(),
            outgoing_timeout_queue: VecDeque::new(),
            timeout_manager: TimeoutManager::new(),

            mode: config.mode,

            current_retry: 0,
            max_retry: config.t2_rty,
            incoming_blocks: VecDeque::new(),
            outgoing_blocks: VecDeque::new(),
            event_queue: VecDeque::new(),
        }
    }

    /// 외부 시스템에 전송할 메시지를 담는다.
    fn emit_event(&mut self, output: Secs1LinkEvent) {
        self.event_queue.push_back(output);
    }

    // 상위로 블록을 보낸다.
    fn emit_block(&mut self, block: Secs1Block) {
        self.outgoing_blocks.push_back(block);
    }

    /// 하위로 데이터를 보낸다.
    fn write(&mut self, data: Vec<u8>) {
        self.outgoing_buffer.extend(data);
    }

    /// timeout을 시작한다.
    fn start_timeout(&mut self, timeout: SecsTimeoutUnit) {
        let ticket = self.timeout_manager.issue(timeout);
        self.outgoing_timeout_queue.push_back(ticket);
    }

    /// timeout을 취소한다.
    fn cancel_timeout(&mut self, timeout: SecsTimeoutUnit) {
        self.timeout_manager.cancel(timeout);
    }

    /** 상태 처리 메서드 영역 */

    /// 상태 머신 로직을 동작한다.
    pub fn run(&mut self) {
        match self.state {
            Secs1LinkState::IDLE => self.handle_idle(),
            Secs1LinkState::LINECONTROL(_) => self.handle_line_control(),
            Secs1LinkState::RECEIVE(_) => self.handle_receive(),
            Secs1LinkState::SEND(_) => self.handle_send(),
            Secs1LinkState::COMPLETION => self.handle_completion(),
        }
    }

    /// IDLE state를 처리한다.
    pub fn handle_idle(&mut self) {
        // 송신 요청한 블록이 없는 상태
        if self.incoming_blocks.is_empty() {
            // LISTEN -> 데이터 수신을 대기한다.
            // ENQ 수신 -> RECEIVE 전환. 아니라면 그냥 버림
            while let Some(byte) = self.incoming_buffer.pop_front() {
                let result = Secs1HandshakeCode::try_from(byte);
                match result {
                    Ok(Secs1HandshakeCode::ENQ) => {
                        self.switch_to_receive(); // ENQ 수신 -> RECEIVE 전환
                        return;
                    }
                    _ => { /* 알 수 없는 신호 -> 무시하고 계속 대기 */ }
                }
            }
        } else {
            // 전송이 필요한 데이터가 존재 -> SEND 전이를 위해 LineControl 진입
            self.switch_to_line_control();
        }
    }

    /// line control state를 처리한다.
    pub fn handle_line_control(&mut self) {
        if let Secs1LinkState::LINECONTROL(line_control_state) = &self.state {
            match line_control_state {
                LineControlState::BeforeSendENQ => {
                    self.write(self.codebytes(Secs1HandshakeCode::ENQ));
                    self.state = Secs1LinkState::LINECONTROL(LineControlState::WaitForResponse);
                    self.start_timeout(SecsTimeoutUnit::T2); // ENQ 송신 후 응답 대기를 위한 T2 timer 시작
                }
                LineControlState::WaitForResponse => {
                    // ENQ 송신 후 상대방의 응답을 기다리는 상태 -> handle_idle과 유사하게 ENQ, EOT 수신 시 처리
                    while let Some(byte) = self.incoming_buffer.pop_front() {
                        if let Ok(code) = Secs1HandshakeCode::try_from(byte) {
                            if self.mode == ConnectionMode::Passive
                                && code == Secs1HandshakeCode::ENQ
                            {
                                // 나는 passive인데 상대에게 ENQ 받음 -> 양보하고 RECEIVE 모드로 전이
                                self.switch_to_receive();
                                return;
                            } else if code == Secs1HandshakeCode::EOT {
                                // EOT를 정상적으로 받아 SEND 모드로 전이
                                self.switch_to_send();
                                return;
                            }
                            // 이외 데이터는 버리고, 계속 반복 진행
                        }
                    }
                }
            }
        } else {
            panic!("invalid state: handle_line_control should only be called in LINECONTROL state");
        }

        // 현재 들어온 데이터에 대한 작업 처리 수행
        // 일반적으로는 아무리 길어도 ms 단위 시간만 소요되므로, timeout 발생 전 처리가 가능할 것으로 기대
        // 정석은 timeout도 while 문 내부에서 체크
    }

    fn retrigger_line_control(&mut self) {
        self.cancel_timeout(SecsTimeoutUnit::T2);
        self.current_retry += 1;

        // FAILED SEND case
        if self.current_retry > self.max_retry {
            // retry 횟수 초과 -> 상위에 timeout 발생 알리고 IDLE 복귀
            self.state = Secs1LinkState::IDLE; // retry 횟수 초과 시 idle 상태로 복귀
            self.current_retry = 0; // retry 0으로 초기화(필수는 아니나, 오류 막기 위한 목적)
            self.emit_event(Secs1LinkEvent::LineControlFailed);
        } else {
            // 상태를 line control로 복구, 타임아웃이 남아 있다면 제거
            self.state = Secs1LinkState::LINECONTROL(LineControlState::BeforeSendENQ);
            self.emit_event(Secs1LinkEvent::Retry(self.current_retry));
        }
    }

    pub fn switch_to_line_control(&mut self) {
        self.current_retry = 0;
        self.state = Secs1LinkState::LINECONTROL(LineControlState::BeforeSendENQ);
    }

    /// send state를 처리한다.
    pub fn handle_send(&mut self) {
        if let Secs1LinkState::SEND(send_state) = &self.state {
            match send_state {
                SendState::BeforeSend => self.process_send_block(),
                SendState::WaitForAck => self.process_wait_for_ack(),
            }
        } else {
            panic!("invalid state: handle_send should only be called in SEND state");
        }
    }

    /// send -> ACK 대기 상태로 전환
    fn process_send_block(&mut self) {
        let (mut bytes, checksum_bytes) = {
            let block = self
                .incoming_blocks
                .front()
                .expect("unexpected empty block queue when send");
            (block.to_bytes(), block.checksum().to_be_bytes())
        };

        bytes.extend_from_slice(&checksum_bytes);
        self.state = Secs1LinkState::SEND(SendState::WaitForAck);
        self.start_timeout(SecsTimeoutUnit::T2);
    }

    /// 데이터 보낸 후 응답 올 때까지 대기
    fn process_wait_for_ack(&mut self) {
        if let Some(byte) = self.incoming_buffer.pop_front() {
            // 일단 timeout을 받았음 -> cancel
            self.cancel_timeout(SecsTimeoutUnit::T2);

            if let Ok(code) = Secs1HandshakeCode::try_from(byte)
                && code == Secs1HandshakeCode::ACK
            {
                // ACK 받음 -> 보낸 블록 제거 + IDLE로 전이
                self.incoming_blocks.pop_front(); // ACK 받았으니 보낸 블록 제거
                self.state = Secs1LinkState::IDLE; // ACK 수신 -> IDLE로 전이
                // BLOCK SENT 상태
                return;
            } else {
                // ACK 아닌 신호 -> 재전송 로직 진입
                self.retrigger_line_control();
            }
        }
    }

    fn switch_to_send(&mut self) {
        self.cancel_timeout(SecsTimeoutUnit::T2);
        self.state = Secs1LinkState::SEND(SendState::BeforeSend); // SEND 모드로 전이
    }

    /// receive state를 처리한다.
    pub fn handle_receive(&mut self) {
        if let Secs1LinkState::RECEIVE(receive_state) = &self.state {
            match receive_state {
                ReceiveState::WaitingLength => self.process_waiting_length(),
                ReceiveState::WaitingData { .. } => self.process_waiting_data(),
                ReceiveState::InvalidBlock => self.process_invalid_block(),
            }
        }
    }

    /// length byte를 수신한다.
    fn process_waiting_length(&mut self) {
        if let Some(byte) = self.incoming_buffer.pop_front() {
            // length byte를 수신한 경우
            self.cancel_timeout(SecsTimeoutUnit::T2);
            self.state = Secs1LinkState::RECEIVE(ReceiveState::WaitingData {
                length: byte,
                buffer: Vec::new(),
            });
            self.start_timeout(SecsTimeoutUnit::T1); // length byte 수신 후 data byte 수신 대기를 위한 T1 timer 시작
        }
    }

    /// length byte를 기반으로 본문 데이터를 기다린다.
    fn process_waiting_data(&mut self) {
        // 데이터가 없는 상태라면 돌아오기
        if self.incoming_buffer.is_empty() {
            return;
        }

        // 데이터가 들어온 상황이므로 초기화
        self.cancel_timeout(SecsTimeoutUnit::T1);
        if let Secs1LinkState::RECEIVE(ReceiveState::WaitingData { length, buffer }) =
            &mut self.state
        {
            while let Some(byte) = self.incoming_buffer.pop_front() {
                buffer.push(byte);

                // 데이터를 정상적으로 수신함
                if buffer.len() == (*length + 2) as usize {
                    let checksum_start = buffer.len() - 2;
                    let (data_part, checksum_part) = buffer.split_at(checksum_start);
                    let expected = u16::from_be_bytes([checksum_part[0], checksum_part[1]]);

                    // 블록 완성 && checksum 검증 성공 -> 상위에 블록 전달 + ACK 전송 + IDLE 복귀
                    if let Ok(block) = Secs1Block::try_from(data_part)
                        && block.verify_checksum(expected)
                    {
                        // 아이템 & ACK 전송
                        self.emit_block(block);
                        self.completion_with_send_ack();
                        return;
                    } else {
                        // 블록 파싱 실패 -> 남은 T1 기간동안 데이터 전부 버린 후 NAK 전송 + IDLE 복귀
                        self.emit_event(Secs1LinkEvent::ReceiveFailed {
                            error: SecsTransportError::InvalidBlock,
                        });
                        self.state = Secs1LinkState::RECEIVE(ReceiveState::InvalidBlock);
                        break;
                    }
                }
            }
        } else {
            panic!(
                "invalid state: process_waiting_data should only be called in WaitingData state"
            );
        }
        self.start_timeout(SecsTimeoutUnit::T1); // 다음 데이터 대기(T1)
    }

    fn process_invalid_block(&mut self) {
        // 들어온 데이터가 없다면 무시하고 돌아감
        if self.incoming_buffer.is_empty() {
            return;
        }

        self.cancel_timeout(SecsTimeoutUnit::T1); // 타임아웃 초기화

        if let Secs1LinkState::RECEIVE(ReceiveState::InvalidBlock) = self.state {
            self.incoming_buffer.clear();
        } else {
            panic!(
                "invalid state: process_invalid_block should only be called in InvalidBlock state"
            );
        }

        self.start_timeout(SecsTimeoutUnit::T1); // 다음 데이터 대기(T1)
    }

    /// IDLE 상태에서 ENQ 수신 시 RECEIVE로 전환하는 로직
    pub fn switch_to_receive(&mut self) {
        self.cancel_timeout(SecsTimeoutUnit::T2);
        self.write(self.codebytes(Secs1HandshakeCode::EOT));
        self.state = Secs1LinkState::RECEIVE(ReceiveState::WaitingLength); // RECEIVE 모드로 전이, length byte는 아직 수신하지 않은 상태
        // EOT 수신 후 length byte 수신 대기를 위한 T2 timer 시작
        self.start_timeout(SecsTimeoutUnit::T2);
        // EOT - length byte
    }

    /// completion state를 처리한다.(경우에 따라 구현 X)
    pub fn handle_completion(&mut self) {}

    /// RECEIVE 중 timeout 발생으로 인한 NAK 전송 및 IDLE 복귀 처리
    pub fn completion_with_send_nak(&mut self, timeout: SecsTimeoutUnit) {
        self.write(self.codebytes(Secs1HandshakeCode::NAK));

        self.emit_event(Secs1LinkEvent::ErrorOccurred(SecsTransportError::Timeout(
            timeout,
        )));

        self.state = Secs1LinkState::IDLE; // NAK 전송 후 IDLE로 복귀
    }

    /// RECEIVE 후 정상 처리로 인한 ACK 전송 및 IDLE 복귀 처리
    pub fn completion_with_send_ack(&mut self) {
        self.write(self.codebytes(Secs1HandshakeCode::ACK));
        self.state = Secs1LinkState::IDLE; // ACK 전송 후 IDLE로 복귀
    }

    /// handshake code를 bytes vector로 변환. into()를 직접 노출하지 않기 위함.
    fn codebytes(&self, code: Secs1HandshakeCode) -> Vec<u8> {
        vec![code.into()]
    }
}

impl Protocol<&[u8], Secs1Block, Secs1LinkEvent> for Secs1LinkMachine {
    type Rout = Secs1Block;
    type Wout = Vec<u8>;
    type Eout = Secs1LinkEvent;
    type Error = SecsTransportError;
    type Time = TimeoutTicket;

    fn handle_read(&mut self, msg: &[u8]) -> Result<(), Self::Error> {
        self.incoming_buffer.extend(msg);
        self.run();
        Ok(())
    }

    fn poll_read(&mut self) -> Option<Self::Rout> {
        self.outgoing_blocks.pop_front()
    }

    fn handle_write(&mut self, msg: Secs1Block) -> Result<(), Self::Error> {
        self.incoming_blocks.push_back(msg);
        Ok(())
    }

    fn poll_write(&mut self) -> Option<Self::Wout> {
        let write = self.outgoing_buffer.drain(..).collect();
        Some(write)
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
        match (&self.state, expired_unit) {
            // LINECONTROL or SEND 상태 & T2 타임아웃
            (Secs1LinkState::LINECONTROL(_), SecsTimeoutUnit::T2)
            | (Secs1LinkState::SEND(_), SecsTimeoutUnit::T2) => self.retrigger_line_control(),

            // RECEIVE 상태 & T2 타임아웃 (WaitingLength)
            (Secs1LinkState::RECEIVE(ReceiveState::WaitingLength), SecsTimeoutUnit::T2) => {
                self.emit_event(Secs1LinkEvent::ReceiveFailed {
                    error: SecsTransportError::Timeout(SecsTimeoutUnit::T2),
                });
                self.completion_with_send_nak(SecsTimeoutUnit::T2);
            }
            // RECEIVE 상태 & T1 타임아웃 (WaitingData or InvalidBlock)
            (Secs1LinkState::RECEIVE(ReceiveState::WaitingData { .. }), SecsTimeoutUnit::T1)
            | (Secs1LinkState::RECEIVE(ReceiveState::InvalidBlock), SecsTimeoutUnit::T1) => {
                self.completion_with_send_nak(SecsTimeoutUnit::T1);
            }
            // 상태와 타임아웃 유닛이 매칭되지 않으면 무시
            _ => {}
        }

        Ok(())
    }

    fn poll_timeout(&mut self) -> Option<Self::Time> {
        self.outgoing_timeout_queue.pop_front()
    }

    fn handle_event(&mut self, _evt: Secs1LinkEvent) -> Result<(), Self::Error> {
        todo!()
        // 외부에서 받을 데이터가 있는지 고민 중.
        // timeout 개념이 존재해서 "즉시" 초기화 필요한 경우가 드물다.
    }

    /// 외부 시스템에서 전송할 메시지를 가져간다.
    fn poll_event(&mut self) -> Option<Self::Eout> {
        self.event_queue.pop_front()
    }
}

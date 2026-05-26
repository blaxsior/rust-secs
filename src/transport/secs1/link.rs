use crate::{
    transport::{
        ConnectionMode,
        error::{SecsTimeoutType, SecsTransportError},
        secs1::{
            block::{Secs1Block, Secs1BlockHeader, Secs1HandshakeCode},
            config::Secs1TransportConfig,
        },
    },
    util::time::TickTimer,
};

use alloc::collections::VecDeque;
use alloc::vec::Vec;
use sans_io_time::Instant;

///
/// SECS-I 통신 block transfer 수행 중 상태
///
enum Secs1LinkState {
    /// 초기 상태
    IDLE,
    /// 전송 방향 수립 중
    LINECONTROL,
    /// 데이터 전송 중
    SEND,
    /// 데이터 수신 중
    RECEIVE(ReceiveState), // 수신한 length byte
    /// 전송 후 종료 단계
    COMPLETION,
}

enum ReceiveState {
    /// length byte 수신 대기
    WaitingLength,
    /// length byte 수신 완료 -> data byte 수신 대기
    WaitingData { length: u8, buffer: Vec<u8> },
    /// block을 파싱했는데 데이터가 이상한 경우 -> 남은 T1 기간동안 데이터 전부 버린 후 NAK 전송 + IDLE 복귀
    InvalidBlock,
}

/// SECS-I Link에서 발생하는 주요 이벤트
#[derive(Debug, PartialEq, Eq)]
pub enum Secs1LinkEvent {
    /// 시리얼 포트로의 전송 요청
    WriteSerial(Vec<u8>),
    /// 상위 계층(Transport)으로 배달할 수신한 SECS-1 블록
    BlockReceived(Secs1Block),

    /// 블록 송신 성공
    SendSuccess { header: Secs1BlockHeader },
    /// 블록 송신 실패
    SendFailed {
        header: Secs1BlockHeader,
        error: SecsTransportError,
    },
    /// Line control 중 문제 발생
    LineControlFailed { error: SecsTransportError },

    /// receive 과정에서 실패
    ReceiveFailed { error: SecsTransportError },
    ///
    ErrorOccurred(SecsTransportError),
}

/// SECS-I block transfer을 처리하는 상태 머신
pub struct Secs1LinkMachine {
    /// SECS-I Link
    state: Secs1LinkState,

    /// serial 수신 된 데이터를 임시 보관하는 큐
    incoming_buffer: VecDeque<u8>,

    /// t1 timeout 처리를 위한 타이머
    t1_timer: TickTimer,
    /// t2 timeout 처리를 위한 타이머
    t2_timer: TickTimer,

    /// SECS-I 통신 모드 (ACTIVE/ PASSIVE)
    mode: ConnectionMode,
    // 현재 retry 횟수
    current_retry: u8,

    // 최대 retry 횟수
    max_retry: u8,

    /// 송신할 블록들을 임시로 저장하는 큐
    pending_blocks: VecDeque<Secs1Block>,
    /// 외부에서 polling으로 꺼내갈 출력 이벤트 큐
    output_queue: VecDeque<Secs1LinkEvent>,
}

impl Secs1LinkMachine {
    pub fn new(config: &Secs1TransportConfig) -> Self {
        Self {
            state: Secs1LinkState::IDLE,
            incoming_buffer: VecDeque::new(),
            t1_timer: TickTimer::new(config.t1_timeout),
            t2_timer: TickTimer::new(config.t2_timeout),
            mode: config.mode,
            current_retry: 0,
            max_retry: config.t2_rty,
            pending_blocks: VecDeque::new(),
            output_queue: VecDeque::new(),
        }
    }

    /// 외부에서 데이터를 읽어 온다.
    pub fn handle_read(&mut self, bytes: &[u8]) {
        self.incoming_buffer.extend(bytes);
    }

    /// 외부 시스템에서 전송이 필요한 메시지를 제출한다.
    pub fn handle_write(&mut self, block: Secs1Block) {
        self.pending_blocks.push_back(block);
    }

    /// 외부 시스템에 전송할 메시지를 담는다.
    fn emit_event(&mut self, output: Secs1LinkEvent) {
        self.output_queue.push_back(output);
    }

    /// 외부 시스템에서 전송할 메시지를 가져간다.
    pub fn poll_event(&mut self) -> Option<Secs1LinkEvent> {
        self.output_queue.pop_front()
    }

    /// 상태 머신 로직을 동작한다.
    pub fn run(&mut self, now: Instant) {
        match self.state {
            Secs1LinkState::IDLE => self.handle_idle(now),
            Secs1LinkState::LINECONTROL => self.handle_line_control(now),
            Secs1LinkState::RECEIVE(_) => self.handle_receive(now),
            Secs1LinkState::SEND => self.handle_send(now),
            Secs1LinkState::COMPLETION => self.handle_completion(now),
        }
    }

    /// IDLE state를 처리한다.
    pub fn handle_idle(&mut self, now: Instant) {
        if self.pending_blocks.is_empty() {
            // LISTEN -> 데이터 수신을 대기한다.
            // ENQ 수신 -> RECEIVE 전환. 아니라면 그냥 버림
            while let Some(first) = self.incoming_buffer.pop_front() {
                let result = Secs1HandshakeCode::try_from(first);
                match result {
                    Ok(Secs1HandshakeCode::ENQ) => {
                        self.switch_to_receive(now); // ENQ 수신 -> RECEIVE 전환
                        return;
                    }
                    _ => { /* 알 수 없는 신호 -> 무시하고 계속 대기 */ }
                }
            }
        } else {
            // 전송이 필요한 데이터가 존재 -> SEND 전이를 위해 LineControl 진입
            self.switch_to_line_control(now);
        }
    }

    /// line control state를 처리한다.
    pub fn handle_line_control(&mut self, now: Instant) {
        // timeout 체크
        if self.t2_timer.check_timeout(now) {
            self.handle_linecontrol_retry(now);
            return; // timeout이 발생한 경우 retry 처리
        }

        // 현재 들어온 데이터에 대한 작업 처리 수행
        // 일반적으로는 아무리 길어도 ms 단위 시간만 소요되므로, timeout 발생 전 처리가 가능할 것으로 기대
        // 정석은 timeout도 while 문 내부에서 체크
        while let Some(byte) = self.incoming_buffer.pop_front() {
            if let Ok(code) = Secs1HandshakeCode::try_from(byte) {
                if code == Secs1HandshakeCode::ENQ && self.mode == ConnectionMode::Passive {
                    // 나는 passive인데 상대에게 ENQ 받음 -> 양보하고 RECEIVE 모드로 전이
                    self.switch_to_receive(now);
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

    fn handle_linecontrol_retry(&mut self, now: Instant) {
        // 기본 상태를 line control로 복구
        self.state = Secs1LinkState::LINECONTROL;
        self.current_retry += 1;
        self.t2_timer.reset();
        self.t2_timer.start(now);

        // FAILED SEND case
        if self.current_retry > self.max_retry {
            // retry
            self.emit_event(Secs1LinkEvent::LineControlFailed {
                error: SecsTransportError::Timeout(SecsTimeoutType::T2),
            });
            self.state = Secs1LinkState::IDLE; // retry 횟수 초과 시 idle 상태로 복귀
            self.current_retry = 0; // retry 0으로 초기화(필수는 아니나, 오류 막기 위한 목적)
        }
    }

    pub fn switch_to_line_control(&mut self, now: Instant) {
        self.state = Secs1LinkState::LINECONTROL;
        self.current_retry = 0;
        self.t2_timer.start(now);
        self.emit_event(Secs1LinkEvent::WriteSerial(
            self.codebytes(Secs1HandshakeCode::ENQ),
        ));
    }

    /// send state를 처리한다.
    pub fn handle_send(&mut self, now: Instant) {
        // if self.state == Secs1LinkState::IDLE {
        //     self.pending_block = Some(block);
        //     self.output_queue.push_back(LinkOutput::WriteSerial(vec![
        //         Secs1HandshakeCode::ENQ.into(),
        //     ])); // ENQ
        //     self.state = Secs1LinkState::LINECONTROL;
        //     self.start_t2();
        // }
    }

    fn switch_to_send(&mut self) {
        self.state = Secs1LinkState::SEND; // SEND 모드로 전이
    }

    /// receive state를 처리한다.
    pub fn handle_receive(&mut self, now: Instant) {
        match self.state {
            Secs1LinkState::RECEIVE(ReceiveState::WaitingLength) => {
                self.process_waiting_length(now)
            }
            Secs1LinkState::RECEIVE(ReceiveState::WaitingData { .. }) => {
                self.process_waiting_data(now)
            }
            Secs1LinkState::RECEIVE(ReceiveState::InvalidBlock) => {
                self.process_invalid_block(now)
            },
            _ => { /* RECEIVE 상태가 아닌 경우는 이 로직에 들어올 수 없으므로 무시 */}
        }
        // 1. length byte 수신 대기
    }

    fn process_waiting_length(&mut self, now: Instant) {
        match self.incoming_buffer.pop_front() {
            // length byte를 수신한 경우
            Some(byte) => {
                self.t2_timer.reset();
                self.state = Secs1LinkState::RECEIVE(ReceiveState::WaitingData {
                    length: byte,
                    buffer: Vec::new(),
                });
            }
            None => {
                // 데이터가 없을 때 타임아웃 체크, T2 timeout이 발생한 경우 NAK 전송 후 IDLE 복귀
                if self.t2_timer.check_timeout(now) {
                    self.completion_with_send_nak(SecsTimeoutType::T2);
                }
            }
        }
    }

    fn process_waiting_data(&mut self, now: Instant) {
        if let Secs1LinkState::RECEIVE(ReceiveState::WaitingData {
            length,
            ref mut buffer,
        }) = self.state
        {
            loop {
                if let Some(byte) = self.incoming_buffer.pop_front() {
                    self.t1_timer.reset();
                    self.t1_timer.start(now);

                    buffer.push(byte);

                    // 데이터를 정상적으로 수신함
                    if buffer.len() == length as usize {
                        self.t1_timer.reset(); // T1 타이머 초기화 (추후 사용 용도)

                        // 블록 완성 -> 상위 계층으로 전달
                        if let Ok(block) = Secs1Block::try_from(buffer.as_slice()) {
                            // ACK 전송 및
                            self.emit_event(Secs1LinkEvent::BlockReceived(block));
                            self.completion_with_send_ack();
                            self.state = Secs1LinkState::IDLE; // 다음 블록 수신을 위해 IDLE로 전이
                        } else {
                            // 블록 파싱 실패 -> 남은 T1 기간동안 데이터 전부 버린 후 NAK 전송 + IDLE 복귀
                            self.state = Secs1LinkState::RECEIVE(ReceiveState::InvalidBlock);
                        }
                        break;
                    }
                } else {
                    // 데이터가 없을 때 타임아웃 체크, T2 timeout이 발생한 경우 NAK 전송 후 IDLE 복귀
                    if self.t1_timer.check_timeout(now) {
                        self.completion_with_send_nak(SecsTimeoutType::T1);
                        break;
                    }
                }
            }
        }
    }

    fn process_invalid_block(&mut self, now: Instant) {
        if let Secs1LinkState::RECEIVE(ReceiveState::InvalidBlock) = self.state {
            while self.incoming_buffer.pop_front().is_some() {
                // 데이터가 들어오는 동안에는 타이머를 계속 리셋해줘야 타임아웃이 안 남
                self.t1_timer.reset();
                self.t1_timer.start(now);
            }

            // 2. 데이터가 다 소진된 경우 타임아웃을 체크
            if self.t1_timer.check_timeout(now) {
                self.completion_with_send_nak(SecsTimeoutType::T1);
            }
        }
    }

    /// IDLE 상태에서 ENQ 수신 시 RECEIVE로 전환하는 로직
    pub fn switch_to_receive(&mut self, now: Instant) {
        self.state = Secs1LinkState::RECEIVE(ReceiveState::WaitingLength); // RECEIVE 모드로 전이, length byte는 아직 수신하지 않은 상태
        self.emit_event(Secs1LinkEvent::WriteSerial(
            self.codebytes(Secs1HandshakeCode::EOT),
        ));
        // EOT 수신 후 length byte 수신 대기를 위한 T2 timer 시작
        self.t2_timer.reset();
        self.t2_timer.start(now);

        // EOT - length byte
    }

    /// completion state를 처리한다.(경우에 따라 구현 X)
    pub fn handle_completion(&mut self, now: Instant) {}

    /// RECEIVE 중 timeout 발생으로 인한 NAK 전송 및 IDLE 복귀 처리
    pub fn completion_with_send_nak(&mut self, timeout: SecsTimeoutType) {
        self.emit_event(Secs1LinkEvent::WriteSerial(
            self.codebytes(Secs1HandshakeCode::NAK),
        ));

        self.emit_event(Secs1LinkEvent::ErrorOccurred(SecsTransportError::Timeout(
            timeout,
        )));
        self.state = Secs1LinkState::IDLE; // NAK 전송 후 IDLE로 복귀
    }

    /// RECEIVE 후 정상 처리로 인한 ACK 전송 및 IDLE 복귀 처리
    pub fn completion_with_send_ack(&mut self) {
        self.emit_event(Secs1LinkEvent::WriteSerial(
            self.codebytes(Secs1HandshakeCode::ACK),
        ));

        self.state = Secs1LinkState::IDLE; // ACK 전송 후 IDLE로 복귀
    }

    /// handshake code를 bytes vector로 변환. into()를 직접 노출하지 않기 위함.
    fn codebytes(&self, code: Secs1HandshakeCode) -> Vec<u8> {
        vec![code.into()]
    }
}

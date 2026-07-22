use crate::{
    transport::{
        ConnectionRole, SecsTimeoutUnit, TimeoutTicket,
        error::SecsTransportError,
        secs1::{
            block::{Secs1Block, Secs1BlockHeader, Secs1HandshakeCode},
            config::Secs1TransportConfig,
        },
    },
    util::time::TimeoutManager,
};

use alloc::collections::VecDeque;
use alloc::vec;
use alloc::vec::Vec;
use sansio::Protocol;

///
/// SECS-I 통신 block transfer 수행 중 상태
///
enum Secs1BlockTransferState {
    /// 초기 상태
    IDLE,
    /// 전송 방향 수립 중
    LINECONTROL(LineControlState),
    /// 데이터 전송 중
    SEND(SendState),
    /// 데이터 수신 중
    RECEIVE(ReceiveState), // 수신한 length byte

                           // / 전송 후 종료 단계. 명세상 존재하나, 각 state에서 바로 IDLE 전이하도록 기능 구현 중
                           // COMPLETION,
}

impl Secs1BlockTransferState {
    pub fn name(&self) -> &'static str {
        match self {
            Secs1BlockTransferState::IDLE => "IDLE",
            Secs1BlockTransferState::LINECONTROL(line_control_state) => match line_control_state {
                LineControlState::BeforeSendENQ => "LINECONTROL:BeforeENQ",
                LineControlState::WaitForResponse => "LINECONTROL:WaitResp",
            },
            Secs1BlockTransferState::SEND(send_state) => match send_state {
                SendState::BeforeSend => "SEND:BeforeSend",
                SendState::WaitForAck => "SEND:WaitForACK",
            },
            Secs1BlockTransferState::RECEIVE(receive_state) => match receive_state {
                ReceiveState::WaitingLength => "RECV:WaitLength",
                ReceiveState::WaitingData { .. } => "RECV:WaitData",
                ReceiveState::InvalidBlock => "RECV:InvalidBlock",
            },
        }
    }
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

/// SECS-I BlockTransfer에서 발생하는 주요 이벤트
#[derive(Debug, PartialEq, Eq)]
pub enum Secs1BlockTransferEvent {
    /// 블록 송신 성공
    SendSuccess { header: Secs1BlockHeader },
    /// 블록 송신 실패
    SendFailed {
        header: Secs1BlockHeader,
        error: SecsTransportError,
    },

    /// Line control 단계 재시도
    Retry(u8),

    /// receive 과정에서 실패
    ReceiveFailed { error: SecsTransportError },
    /// 기타 오류
    ErrorOccurred(SecsTransportError),
}

/// SECS-I block transfer을 처리하는 상태 머신
pub struct Secs1BlockTransferMachine {
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
    outgoing_event_queue: VecDeque<Secs1BlockTransferEvent>,

    timeout_manager: TimeoutManager,

    /** 내부 상태 값 */
    /// SECS-I block transfer current state
    state: Secs1BlockTransferState,
    /// SECS-I 통신 모드 (ACTIVE/ PASSIVE)
    mode: ConnectionRole,
    // 현재 retry 횟수
    current_retry: u8,
    // 최대 retry 횟수
    max_retry: u8,

    // 상태 전이 후 동작을 수행하는 것이 목적
    process_next: bool,
}

impl Secs1BlockTransferMachine {
    pub fn new(config: &Secs1TransportConfig) -> Self {
        Self {
            state: Secs1BlockTransferState::IDLE,

            incoming_buffer: VecDeque::new(),
            outgoing_buffer: VecDeque::new(),
            outgoing_timeout_queue: VecDeque::new(),
            timeout_manager: TimeoutManager::new(),

            mode: config.local_role,

            current_retry: 0,
            max_retry: config.t2_rty_limit,
            incoming_blocks: VecDeque::new(),
            outgoing_blocks: VecDeque::new(),
            outgoing_event_queue: VecDeque::new(),
            process_next: true,
        }
    }

    fn next(&mut self) {
        self.process_next = true;
    }

    /// 외부 시스템에 전송할 메시지를 담는다.
    fn emit_event(&mut self, output: Secs1BlockTransferEvent) {
        self.outgoing_event_queue.push_back(output);
    }

    /// 상위로 블록을 보낸다.
    fn emit_block(&mut self, block: Secs1Block) {
        self.outgoing_blocks.push_back(block);
    }

    /// 하위로 데이터를 보낸다.
    fn write(&mut self, data: Vec<u8>) {
        self.outgoing_buffer.extend(data);
    }

    /// timeout을 시작한다.
    fn start_timeout(&mut self, timeout: SecsTimeoutUnit) {
        log::debug!("[{}] start timeout {:?}", self.state.name(), timeout);
        let ticket = self.timeout_manager.issue(timeout);
        self.outgoing_timeout_queue.push_back(ticket);
    }

    /// timeout을 취소한다.
    fn cancel_timeout(&mut self, timeout: SecsTimeoutUnit) {
        log::debug!("[{}] cancel timeout {:?}", self.state.name(), timeout);
        self.timeout_manager.cancel(timeout);
    }

    fn state_change(&mut self, state: Secs1BlockTransferState) {
        self.state = state;
        self.next();
    }

    /** 상태 처리 메서드 영역 */

    /// 상태 머신 로직을 동작한다.
    pub fn run(&mut self) {
        // 초기 호출 시 작업 진행
        self.process_next = true;

        while self.process_next {
            self.process_next = false;
            match self.state {
                Secs1BlockTransferState::IDLE => self.handle_idle(),
                Secs1BlockTransferState::LINECONTROL(_) => self.handle_line_control(),
                Secs1BlockTransferState::RECEIVE(_) => self.handle_receive(),
                Secs1BlockTransferState::SEND(_) => self.handle_send(),
                // Secs1BlockTransferState::COMPLETION => self.handle_completion(),
            }
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
            log::debug!("[{}] switch to line control", self.state.name());
            self.switch_to_line_control();
        }
    }

    /// line control state를 처리한다.
    pub fn handle_line_control(&mut self) {
        let Secs1BlockTransferState::LINECONTROL(line_control_state) = &self.state else {
            panic!("invalid state: handle_line_control should only be called in LINECONTROL state");
        };

        match line_control_state {
            LineControlState::BeforeSendENQ => {
                log::debug!("[{}] send ENQ, switch to recv", self.state.name());
                self.write(self.codebytes(Secs1HandshakeCode::ENQ));
                self.start_timeout(SecsTimeoutUnit::T2); // ENQ 송신 후 응답 대기를 위한 T2 timer 시작
                self.state_change(Secs1BlockTransferState::LINECONTROL(
                    LineControlState::WaitForResponse,
                ));
            }
            LineControlState::WaitForResponse => {
                // ENQ 송신 후 상대방의 응답을 기다리는 상태 -> handle_idle과 유사하게 ENQ, EOT 수신 시 처리
                while let Some(byte) = self.incoming_buffer.pop_front() {
                    let Ok(code) = Secs1HandshakeCode::try_from(byte) else {
                        continue;
                    };

                    if self.mode == ConnectionRole::Passive && code == Secs1HandshakeCode::ENQ {
                        log::debug!("[{}] passive + ENQ, switch to RECV", self.state.name());
                        // 나는 passive인데 상대에게 ENQ 받음 -> 양보하고 RECEIVE 모드로 전이
                        self.switch_to_receive();
                        return;
                    } else if code == Secs1HandshakeCode::EOT {
                        // EOT를 정상적으로 받아 SEND 모드로 전이
                        log::debug!("[{}] active + EOT, switch to SEND", self.state.name());
                        self.switch_to_send();
                        return;
                    }
                    // 이외 데이터는 버리고, 계속 반복 진행
                }
            }
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
            self.current_retry = 0; // retry 0으로 초기화(필수는 아니나, 오류 막기 위한 목적)
            self.state_change(Secs1BlockTransferState::IDLE);
            let block = self.pop_incoming_block();
            self.emit_event(Secs1BlockTransferEvent::SendFailed {
                header: block.header,
                error: SecsTransportError::Timeout(SecsTimeoutUnit::T2),
            });
        } else {
            // 상태를 line control로 복구, 타임아웃이 남아 있다면 제거
            self.state_change(Secs1BlockTransferState::LINECONTROL(
                LineControlState::BeforeSendENQ,
            ));
            self.emit_event(Secs1BlockTransferEvent::Retry(self.current_retry));
        }
    }

    pub fn switch_to_line_control(&mut self) {
        self.current_retry = 0;
        self.state_change(Secs1BlockTransferState::LINECONTROL(
            LineControlState::BeforeSendENQ,
        ));
    }

    /// send state를 처리한다.
    pub fn handle_send(&mut self) {
        let Secs1BlockTransferState::SEND(send_state) = &self.state else {
            panic!("invalid state: handle_send should only be called in SEND state");
        };

        match send_state {
            SendState::BeforeSend => self.process_send_block(),
            SendState::WaitForAck => self.process_wait_for_ack(),
        }
    }

    fn pop_incoming_block(&mut self) -> Secs1Block {
        self.incoming_blocks.pop_front().expect("block must exists")
    }

    /// send -> ACK 대기 상태로 전환
    fn process_send_block(&mut self) {
        let bytes = {
            let block = self
                .incoming_blocks
                .front()
                .expect("unexpected empty block queue when send");
            block.to_bytes_with_checksum()
        };

        self.write(bytes);
        self.start_timeout(SecsTimeoutUnit::T2);
        self.state_change(Secs1BlockTransferState::SEND(SendState::WaitForAck));
    }

    /// 데이터 보낸 후 응답 올 때까지 대기
    fn process_wait_for_ack(&mut self) {
        let Some(byte) = self.incoming_buffer.pop_front() else {
            return;
        };

        // 일단 timeout을 받았음 -> cancel
        self.cancel_timeout(SecsTimeoutUnit::T2);

        if let Ok(code) = Secs1HandshakeCode::try_from(byte)
            && code == Secs1HandshakeCode::ACK
        {
            // ACK 받음 -> 보낸 블록 제거 + IDLE로 전이
            let block = self.pop_incoming_block();
            self.emit_event(Secs1BlockTransferEvent::SendSuccess {
                header: block.header,
            });
            self.state_change(Secs1BlockTransferState::IDLE); // ACK 수신 -> IDLE로 전이
            // BLOCK SENT 상태
            return;
        } else {
            // ACK 아닌 신호 -> 재전송 로직 진입
            self.retrigger_line_control();
        }
    }

    fn switch_to_send(&mut self) {
        self.cancel_timeout(SecsTimeoutUnit::T2);
        self.state_change(Secs1BlockTransferState::SEND(SendState::BeforeSend));
    }

    /// receive state를 처리한다.
    pub fn handle_receive(&mut self) {
        let Secs1BlockTransferState::RECEIVE(receive_state) = &self.state else {
            return;
        };

        match receive_state {
            ReceiveState::WaitingLength => self.process_waiting_length(),
            ReceiveState::WaitingData { .. } => self.process_waiting_data(),
            ReceiveState::InvalidBlock => self.process_invalid_block(),
        }
    }

    /// length byte를 수신한다.
    fn process_waiting_length(&mut self) {
        let Some(byte) = self.incoming_buffer.pop_front() else {
            return;
        };
        // length byte를 수신한 경우
        self.cancel_timeout(SecsTimeoutUnit::T2);
        log::debug!("[{}] length byte received", self.state.name());

        let is_length_valid = byte >= 10 && byte <= 254;
        if is_length_valid {
            // 정상 length 수신한 경우 -> data 수신 모드로 전이
            self.state_change(Secs1BlockTransferState::RECEIVE(
                ReceiveState::WaitingData {
                    length: byte,
                    buffer: Vec::new(),
                },
            ));
        } else {
            // length가 비정상 -> 데이터 drop 모드로 전이
            self.state_change(Secs1BlockTransferState::RECEIVE(ReceiveState::InvalidBlock));
        }
        self.start_timeout(SecsTimeoutUnit::T1); // length byte 수신 후 data byte 수신 대기를 위한 T1 timer 시작
    }

    /// length byte를 기반으로 본문 데이터를 기다린다.
    fn process_waiting_data(&mut self) {
        // 데이터가 없는 상태라면 돌아오기
        if self.incoming_buffer.is_empty() {
            return;
        }

        // 데이터가 들어온 상황이므로 초기화
        self.cancel_timeout(SecsTimeoutUnit::T1);

        let Secs1BlockTransferState::RECEIVE(ReceiveState::WaitingData { length, buffer }) =
            &mut self.state
        else {
            panic!("invalid state: process_waiting_data only called in InvalidBlock state");
        };
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
                    log::debug!(
                        "[{}] success to recv block. {:?}",
                        self.state.name(),
                        block.header
                    );

                    self.emit_block(block);
                    self.completion_with_send_ack();
                    log::debug!("[{}] send ACK, return to idle", self.state.name());
                    return;
                } else {
                    // 블록 파싱 실패 -> 남은 T1 기간동안 데이터 전부 버린 후 NAK 전송 + IDLE 복귀
                    self.emit_event(Secs1BlockTransferEvent::ReceiveFailed {
                        error: SecsTransportError::BlockError,
                    });
                    log::warn!(
                        "[{}] something wrong when parsing block... skip next bytes",
                        self.state.name()
                    );
                    self.state_change(Secs1BlockTransferState::RECEIVE(ReceiveState::InvalidBlock));
                    break;
                }
            }
        }
        self.start_timeout(SecsTimeoutUnit::T1); // 다음 데이터 대기(T1)
    }

    fn process_invalid_block(&mut self) {
        // 들어온 데이터가 없다면 무시하고 돌아감
        if self.incoming_buffer.is_empty() {
            return;
        }

        self.cancel_timeout(SecsTimeoutUnit::T1); // 타임아웃 초기화

        let Secs1BlockTransferState::RECEIVE(ReceiveState::InvalidBlock) = self.state else {
            panic!("invalid state: process_invalid_block only called in InvalidBlock state");
        };

        self.incoming_buffer.clear();
        log::warn!(
            "[{}] invalid block state. clear incoming buffer",
            self.state.name()
        );

        self.start_timeout(SecsTimeoutUnit::T1); // 다음 데이터 대기(T1)
    }

    /// IDLE 상태에서 ENQ 수신 시 RECEIVE로 전환하는 로직
    pub fn switch_to_receive(&mut self) {
        self.cancel_timeout(SecsTimeoutUnit::T2);
        self.write(self.codebytes(Secs1HandshakeCode::EOT));

        // EOT 수신 후 length byte 수신 대기를 위한 T2 timer 시작
        self.start_timeout(SecsTimeoutUnit::T2);
        // RECEIVE 모드로 전이, length byte는 아직 수신하지 않은 상태
        self.state_change(Secs1BlockTransferState::RECEIVE(
            ReceiveState::WaitingLength,
        ));
    }

    /// completion state를 처리한다.(경우에 따라 구현 X)
    // pub fn handle_completion(&mut self) {}

    /// RECEIVE 중 timeout 발생으로 인한 NAK 전송 및 IDLE 복귀 처리
    pub fn completion_with_send_nak(&mut self) {
        self.write(self.codebytes(Secs1HandshakeCode::NAK));
        log::debug!("[{}] send nak, return to idle", self.state.name());
        self.state_change(Secs1BlockTransferState::IDLE); // NAK 전송 후 IDLE로 복귀
    }

    /// RECEIVE 후 정상 처리로 인한 ACK 전송 및 IDLE 복귀 처리
    pub fn completion_with_send_ack(&mut self) {
        self.write(self.codebytes(Secs1HandshakeCode::ACK));
        log::debug!("[{}] send ack, return to idle", self.state.name());
        self.state_change(Secs1BlockTransferState::IDLE); // ACK 전송 후 IDLE로 복귀
    }

    /// handshake code를 bytes vector로 변환. into()를 직접 노출하지 않기 위함.
    fn codebytes(&self, code: Secs1HandshakeCode) -> Vec<u8> {
        vec![code.into()]
    }
}

impl Protocol<&[u8], Secs1Block, ()> for Secs1BlockTransferMachine {
    type Rout = Secs1Block;
    type Wout = Vec<u8>;
    type Eout = Secs1BlockTransferEvent;
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
        self.run();
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

        // 2. 상태(State)와 유닛(Unit)을 튜플로 묶어서 매칭
        match (&self.state, expired_unit) {
            // LINECONTROL or SEND 상태 & T2 타임아웃
            (Secs1BlockTransferState::LINECONTROL(..), SecsTimeoutUnit::T2) => {
                log::warn!("[{}] T2 timeout.", self.state.name());
                self.retrigger_line_control()
            }

            (Secs1BlockTransferState::SEND(..), SecsTimeoutUnit::T2) => {
                log::warn!("[{}] T2 timeout.", self.state.name());
                self.retrigger_line_control()
            }

            // RECEIVE 상태 & T2 타임아웃 (WaitingLength)
            (
                Secs1BlockTransferState::RECEIVE(ReceiveState::WaitingLength),
                SecsTimeoutUnit::T2,
            ) => {
                self.emit_event(Secs1BlockTransferEvent::ReceiveFailed {
                    error: SecsTransportError::Timeout(SecsTimeoutUnit::T2),
                });
                log::warn!("[{}] T2 timeout.", self.state.name());
                self.completion_with_send_nak();
            }
            // RECEIVE 상태 & T1 타임아웃 (WaitingData or InvalidBlock)
            (
                Secs1BlockTransferState::RECEIVE(ReceiveState::WaitingData { .. }),
                SecsTimeoutUnit::T1,
            ) => {
                self.emit_event(Secs1BlockTransferEvent::ReceiveFailed {
                    error: SecsTransportError::Timeout(SecsTimeoutUnit::T1),
                });
                log::warn!("[{}] T1 timeout.", self.state.name());
                self.completion_with_send_nak();
            }

            // RECEIVE InvalidBlock 중 Timeout
            (Secs1BlockTransferState::RECEIVE(ReceiveState::InvalidBlock), SecsTimeoutUnit::T1) => {
                self.emit_event(Secs1BlockTransferEvent::ReceiveFailed {
                    error: SecsTransportError::BlockError,
                });
                log::warn!("[{}] T1 timeout.", self.state.name());
                self.completion_with_send_nak();
            }
            // 상태와 타임아웃 유닛이 매칭되지 않으면 무시
            _ => {}
        }
        // 상태 전이에 따른 작업 수행
        self.run();
        Ok(())
    }

    fn poll_timeout(&mut self) -> Option<Self::Time> {
        self.outgoing_timeout_queue.pop_front()
    }

    fn handle_event(&mut self, _evt: ()) -> Result<(), Self::Error> {
        // 별도 구현 없음
        Ok(())
    }

    /// 외부 시스템에서 전송할 메시지를 가져간다.
    fn poll_event(&mut self) -> Option<Self::Eout> {
        self.outgoing_event_queue.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use sansio::Protocol;
    use secs_ii::{FunctionId, StreamId};

    use crate::transport::{
        ConnectionRole, DeviceId, Rbit, SecsTimeoutUnit, SystemByte, Wbit,
        secs1::{
            block::{Secs1Block, Secs1BlockHeader, Secs1HandshakeCode},
            config::Secs1TransportConfig,
            protocol::block_transfer::{
                LineControlState, ReceiveState, Secs1BlockTransferMachine, Secs1BlockTransferState,
                SendState,
            },
        },
    };

    fn config_active() -> Secs1TransportConfig {
        Secs1TransportConfig {
            device_id: DeviceId(1),
            local_role: ConnectionRole::Active,
            t1_timeout: Duration::from_millis(1000),
            t2_timeout: Duration::from_millis(1000),
            t3_timeout: Duration::from_millis(1000),
            t4_timeout: Duration::from_millis(1000),
            t2_rty_limit: 2,
        }
    }

    fn config_passive() -> Secs1TransportConfig {
        Secs1TransportConfig {
            device_id: DeviceId(1),
            local_role: ConnectionRole::Passive,
            t1_timeout: Duration::from_millis(1000),
            t2_timeout: Duration::from_millis(1000),
            t3_timeout: Duration::from_millis(1000),
            t4_timeout: Duration::from_millis(1000),
            t2_rty_limit: 2,
        }
    }

    fn message_block(device_id: DeviceId) -> Secs1Block {
        let header = Secs1BlockHeader {
            device_id: device_id,
            rbit: Rbit(false),
            wbit: Wbit(true),
            stream: StreamId(1),
            function: FunctionId(3),
            ebit: true,
            block_no: 1,
            system_byte: SystemByte(1),
        };

        let block = Secs1Block {
            header,
            data: vec![0x01, 0x01, 0xB1, 0x04, 0x00, 0x00, 0x0B, 0xB8],
        };

        block
    }

    // IDLE -> LINE CONTROL 전이(send 아이템 존재하는 경우)
    #[test]
    fn test_idle_to_line_control() {
        let config = config_active();
        // 1. 머신 초기화
        let mut machine = Secs1BlockTransferMachine::new(&config);
        assert!(matches!(machine.state, Secs1BlockTransferState::IDLE));

        let block = message_block(config.device_id);

        // 3. handle_write 호출 (내부적으로 incoming_blocks에 넣고 run() 호출)
        machine.handle_write(block).unwrap();

        // 4. 상태 전이 검증
        // 기대하는 상태: LINECONTROL -> WaitResp
        assert!(matches!(
            machine.state,
            Secs1BlockTransferState::LINECONTROL(LineControlState::WaitForResponse)
        ));

        // 5. 추가 검증: outgoing_buffer에 ENQ가 들어갔는지 확인
        // handle_line_control이 실행되면서 write(ENQ)가 호출되었을 것임
        let output = machine.poll_write();
        assert!(output.is_some());
        assert_eq!(output.unwrap(), vec![Secs1HandshakeCode::ENQ.into()]);
    }

    // IDLE -> RECV 전이: nothing to send + ENQ recv
    #[test]
    fn test_idle_to_recv() {
        let config = config_active();
        // 1. 머신 초기화
        let mut machine = Secs1BlockTransferMachine::new(&config);
        assert!(matches!(machine.state, Secs1BlockTransferState::IDLE));

        // 아이템이 없는 상태에서 ENQ 수신 -> RECV 전이
        machine
            .handle_read(&[Secs1HandshakeCode::ENQ.into()])
            .unwrap();

        assert!(matches!(
            machine.state,
            Secs1BlockTransferState::RECEIVE(ReceiveState::WaitingLength)
        ));

        // T2 timeout 발생
        let ticket = machine.poll_timeout().unwrap();
        assert!(matches!(ticket.timeout, SecsTimeoutUnit::T2));
    }

    // Active가 LINE CONTROL 시 EOT 받음 -> 양보 X
    #[test]
    fn test_line_control_active_recv_enq() {
        let config = config_active();
        // 1. 머신 초기화
        let mut machine = Secs1BlockTransferMachine::new(&config);
        machine.state = Secs1BlockTransferState::LINECONTROL(LineControlState::WaitForResponse);

        machine
            .handle_read(&[Secs1HandshakeCode::ENQ.into()])
            .unwrap();

        // active 모드의 경우 ENQ 받아도 전이 X 그냥 무시
        assert!(matches!(
            machine.state,
            Secs1BlockTransferState::LINECONTROL(LineControlState::WaitForResponse)
        ));
    }

    // Passive가 LINE CONTROL 시 EOT 받음 -> 양보, RECV 전이
    #[test]
    fn test_line_control_passive_recv_enq_then_move_to_recv() {
        let config = config_passive();

        let mut machine = Secs1BlockTransferMachine::new(&config);
        machine.state = Secs1BlockTransferState::LINECONTROL(LineControlState::WaitForResponse);

        machine
            .handle_read(&[Secs1HandshakeCode::ENQ.into()])
            .unwrap();

        // length byte 대기 모드로 전이
        assert!(matches!(
            machine.state,
            Secs1BlockTransferState::RECEIVE(ReceiveState::WaitingLength)
        ));

        // EOT 전송
        let bytes = machine.poll_write();
        assert!(bytes.is_some());
        assert_eq!(bytes.unwrap(), vec![Secs1HandshakeCode::EOT.into()]);

        // T2 timeout 시작
        let timeout = machine.poll_timeout();
        assert!(timeout.is_some());
        assert_eq!(timeout.unwrap().timeout, SecsTimeoutUnit::T2);
    }

    // LINE CONTROL 중 retry 초과 -> IDLE 전이
    #[test]
    fn test_line_control_fail_send_move_to_idle() {
        let config = config_active();
        let mut machine = Secs1BlockTransferMachine::new(&config);
        let block = message_block(config.device_id);
        machine.handle_write(block).unwrap();

        for rty in 0..(config.t2_rty_limit + 1) {
            assert!(matches!(
                machine.state,
                Secs1BlockTransferState::LINECONTROL(LineControlState::WaitForResponse)
            ));
            assert_eq!(machine.current_retry, rty);
            let output = machine.poll_write().expect("expect bytes but found None");
            assert_eq!(output, vec![Secs1HandshakeCode::ENQ.into()]);
            let ticket = machine
                .poll_timeout()
                .expect("expect timout but found None");
            assert_eq!(ticket.timeout, SecsTimeoutUnit::T2);
            machine.handle_timeout(ticket).unwrap();
        }

        // limit + 1 회의 timeout 발생, idle 모드로 복귀
        assert_eq!(machine.current_retry, 0);
        assert!(matches!(machine.state, Secs1BlockTransferState::IDLE));
    }

    // SEND mode ACK 수신 -> IDLE 전이
    #[test]
    fn test_send_success_move_to_idle() {
        let config = config_active();
        let mut machine = Secs1BlockTransferMachine::new(&config);
        let block = message_block(config.device_id);

        // send block -> line control
        machine.handle_write(block).unwrap();
        assert!(matches!(
            machine.state,
            Secs1BlockTransferState::LINECONTROL(LineControlState::WaitForResponse)
        ));
        let ticket = machine
            .poll_timeout()
            .expect("expect timout but found None");
        assert_eq!(ticket.timeout, SecsTimeoutUnit::T2);

        // recv EOT -> move to send
        machine
            .handle_read(&[Secs1HandshakeCode::EOT.into()])
            .unwrap();
        assert!(matches!(
            machine.state,
            Secs1BlockTransferState::SEND(SendState::WaitForAck)
        ));
        let bytes = machine.poll_write().expect("expect block bytes but None");
        assert!(bytes.len() > 0);
        let ticket = machine
            .poll_timeout()
            .expect("expect timout but found None");
        assert_eq!(ticket.timeout, SecsTimeoutUnit::T2);

        // recv ack after send -> move to idle
        machine
            .handle_read(&[Secs1HandshakeCode::ACK.into()])
            .unwrap();
        assert!(matches!(machine.state, Secs1BlockTransferState::IDLE));
        // return to idle
    }

    // SEND mode NAK 수신 -> LINE CONTROL 전이
    #[test]
    fn test_send_fail_with_nak_move_to_line_control() {
        let config = config_active();
        let mut machine = Secs1BlockTransferMachine::new(&config);
        let block = message_block(config.device_id);

        // send block -> line control
        machine.handle_write(block).unwrap();
        assert!(matches!(
            machine.state,
            Secs1BlockTransferState::LINECONTROL(LineControlState::WaitForResponse)
        ));
        let ticket = machine
            .poll_timeout()
            .expect("expect timout but found None");
        assert_eq!(ticket.timeout, SecsTimeoutUnit::T2);

        // recv EOT -> move to send
        machine
            .handle_read(&[Secs1HandshakeCode::EOT.into()])
            .unwrap();
        assert!(matches!(
            machine.state,
            Secs1BlockTransferState::SEND(SendState::WaitForAck)
        ));
        let bytes = machine.poll_write().expect("expect block bytes but None");
        assert!(bytes.len() > 0);
        let ticket = machine
            .poll_timeout()
            .expect("expect timout but found None");
        assert_eq!(ticket.timeout, SecsTimeoutUnit::T2);

        // recv nak after send -> move to line control
        machine
            .handle_read(&[Secs1HandshakeCode::NAK.into()])
            .unwrap();
        assert!(matches!(
            machine.state,
            Secs1BlockTransferState::LINECONTROL(LineControlState::WaitForResponse)
        ));
        assert_eq!(machine.current_retry, 1);
        // return to idle
    }

    // SEND mode t2 timeout 발생 -> LINE CONTROL 전이
    #[test]
    fn test_send_fail_with_t2_timeout_move_to_line_control() {
        let config = config_active();
        let mut machine = Secs1BlockTransferMachine::new(&config);
        let block = message_block(config.device_id);

        // send block -> line control
        machine.handle_write(block).unwrap();
        assert!(matches!(
            machine.state,
            Secs1BlockTransferState::LINECONTROL(LineControlState::WaitForResponse)
        ));
        let ticket = machine
            .poll_timeout()
            .expect("expect timout but found None");
        assert_eq!(ticket.timeout, SecsTimeoutUnit::T2);

        // recv EOT -> move to send
        machine
            .handle_read(&[Secs1HandshakeCode::EOT.into()])
            .unwrap();
        assert!(matches!(
            machine.state,
            Secs1BlockTransferState::SEND(SendState::WaitForAck)
        ));
        let bytes = machine.poll_write().expect("expect block bytes but None");
        assert!(bytes.len() > 0);
        let ticket = machine
            .poll_timeout()
            .expect("expect timout but found None");
        assert_eq!(ticket.timeout, SecsTimeoutUnit::T2);

        // timeout 발생
        machine.handle_timeout(ticket).unwrap();
        // line control 재진입
        assert!(matches!(
            machine.state,
            Secs1BlockTransferState::LINECONTROL(LineControlState::WaitForResponse)
        ));
        assert_eq!(machine.current_retry, 1);
        // return to idle
    }

    // recv - wait length 중 timeout 발생한 경우 -> move to Idle / send NAK
    #[test]
    fn test_recv_t2_timeout_then_send_nak_move_to_idle() {
        let config = config_passive();

        let mut machine = Secs1BlockTransferMachine::new(&config);
        machine.state = Secs1BlockTransferState::LINECONTROL(LineControlState::WaitForResponse);

        machine
            .handle_read(&[Secs1HandshakeCode::ENQ.into()])
            .unwrap();

        // length byte 대기 모드로 전이
        assert!(matches!(
            machine.state,
            Secs1BlockTransferState::RECEIVE(ReceiveState::WaitingLength)
        ));

        // EOT 전송
        let bytes = machine.poll_write().unwrap();
        assert_eq!(bytes, vec![Secs1HandshakeCode::EOT.into()]);

        // T2 timeout 시작
        let ticket = machine.poll_timeout().unwrap();
        assert_eq!(ticket.timeout, SecsTimeoutUnit::T2);

        // T2 timeout 발생
        machine.handle_timeout(ticket).unwrap();

        // NAK 전송
        let bytes = machine.poll_write().unwrap();
        assert_eq!(bytes, vec![Secs1HandshakeCode::NAK.into()]);

        // IDLE 전이
        assert!(matches!(machine.state, Secs1BlockTransferState::IDLE));
    }

    // length byte 수신했으나 잘못된 byte인 경우
    #[test]
    fn test_recv_invalid_length_then_move_to_recv_invalidblock() {
        let config = config_passive();

        let mut machine = Secs1BlockTransferMachine::new(&config);
        machine.state = Secs1BlockTransferState::LINECONTROL(LineControlState::WaitForResponse);

        machine
            .handle_read(&[Secs1HandshakeCode::ENQ.into()])
            .unwrap();

        // length byte 대기 모드로 전이
        assert!(matches!(
            machine.state,
            Secs1BlockTransferState::RECEIVE(ReceiveState::WaitingLength)
        ));

        // EOT 전송
        let bytes = machine.poll_write().unwrap();
        assert_eq!(bytes, vec![Secs1HandshakeCode::EOT.into()]);

        // T2 timeout 시작
        let ticket = machine.poll_timeout().unwrap();
        assert_eq!(ticket.timeout, SecsTimeoutUnit::T2);

        // 10 <= length <= 254 -> 잘못된 length byte가 들어온 경우
        machine.handle_read(&[255]).unwrap();

        // invalid block mode 전이
        assert!(matches!(
            machine.state,
            Secs1BlockTransferState::RECEIVE(ReceiveState::InvalidBlock)
        ));

        // t1 timeout 시작
        let ticket = machine.poll_timeout().unwrap();
        assert_eq!(ticket.timeout, SecsTimeoutUnit::T1);
    }

    // invalid block 상태인 경우 입력되는 데이터를 계속 버리다가 T1 timeout 발생 시 nak 전송하고 idle 전이
    #[test]
    fn test_recv_invalid_block_discard_blocks_and_move_to_idle_with_nak() {
        let config = config_passive();

        let mut machine = Secs1BlockTransferMachine::new(&config);
        machine.state = Secs1BlockTransferState::RECEIVE(ReceiveState::InvalidBlock);

        machine.handle_read(&[0x00, 0x00]).unwrap();

        // T1 timeout 시작
        let ticket = machine.poll_timeout().unwrap();
        assert_eq!(ticket.timeout, SecsTimeoutUnit::T1);

        // T1 timeout 발생
        machine.handle_timeout(ticket).unwrap();

        // NAK 전송
        let bytes = machine.poll_write().unwrap();
        assert_eq!(bytes, vec![Secs1HandshakeCode::NAK.into()]);

        // IDLE 전이
        assert!(matches!(machine.state, Secs1BlockTransferState::IDLE));
    }

    // data 수신 중 t1 timeout 이 발생한 경우 NAK 전송
    #[test]
    fn test_recv_data_t1_timeout_then_move_to_idle_with_nak() {
        let config = config_passive();

        let mut machine = Secs1BlockTransferMachine::new(&config);
        machine.state = Secs1BlockTransferState::RECEIVE(ReceiveState::WaitingData {
            length: 2,
            buffer: Vec::new(),
        });
        // 첫 byte 받음
        machine.handle_read(&[0x00]).unwrap();

        // T1 timeout 시작
        let ticket = machine.poll_timeout().unwrap();
        assert_eq!(ticket.timeout, SecsTimeoutUnit::T1);

        // T1 timeout 발생
        machine.handle_timeout(ticket).unwrap();

        // NAK 전송
        let bytes = machine.poll_write().unwrap();
        assert_eq!(bytes, vec![Secs1HandshakeCode::NAK.into()]);

        // IDLE 전이
        assert!(matches!(machine.state, Secs1BlockTransferState::IDLE));
    }

    // 정상적으로 block 송신한 경우 ack 전송 & idle 전이
    #[test]
    fn test_recv_success_recv_then_move_idle_with_ack() {
        let config = config_passive();

        let mut machine = Secs1BlockTransferMachine::new(&config);
        // header 10 + data 8 => 18
        machine.state = Secs1BlockTransferState::RECEIVE(ReceiveState::WaitingData {
            length: 18,
            buffer: Vec::new(),
        });

        let expected = message_block(config.device_id);
        let bytes = expected.to_bytes_with_checksum();
        //secs block 읽기

        machine.handle_read(&bytes).unwrap();

        println!("state = {}", machine.state.name());

        // ACK 전송
        let bytes = machine.poll_write().unwrap();
        assert_eq!(bytes, vec![Secs1HandshakeCode::ACK.into()]);

        // IDLE 전이
        assert!(matches!(machine.state, Secs1BlockTransferState::IDLE));

        let block = machine.poll_read().unwrap();
        assert_eq!(block.data, expected.data);
        assert_eq!(block.header, expected.header);
    }
}

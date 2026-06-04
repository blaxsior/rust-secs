use crate::{
    transport::{
        ConnectionMode,
        error::{SecsTimeoutUnit, SecsTransportError},
        secs1::{
            block::{Secs1Block, Secs1BlockHeader},
            config::{DeviceId, Secs1TransportConfig},
        },
    },
    util::time::{TimeoutManager, TimeoutTicket},
};

use alloc::collections::VecDeque;
use alloc::vec::Vec;
use sansio::Protocol;
use secs_ii::item::{Secs2Item, Secs2Variant};

/// secs-i message protocol 수행 중 발생하는 이벤트
pub enum Secs1MessageProtocolEvent {
    /// 블록을 성공적으로 송신
    BlockSent { header: Secs1BlockHeader },

}

pub enum Secs1MessageState {
    Idle,
    /// Message 수신 중 (T4 대기)
    Receiving {
        expected_header: Secs1BlockHeader,
        blocks: Vec<u8>,
    },
    /// Primary 전송 후 Secondary Message 대기 중 (T3 대기)
    WaitingForReply {
        expected_header: Secs1BlockHeader,
    },
}

pub struct Secs1MessageProtocolMachine {
    /** 상태 임시 저장 */

    /// block 수신 된 데이터를 임시 보관하는 큐. handle_read
    incoming_buffer: VecDeque<Secs1Block>,
    /// block 송신할 데이터를 임시로 보관하는 큐. poll_write
    outgoing_buffer: VecDeque<Secs1Block>,
    /// secs-ii msg 송신 요청한 블록들을 임시로 저장하는 큐. handle_write
    incoming_blocks: VecDeque<Secs2Variant>,
    /// secs-ii msg 수신한 블록들을 임시로 저장하는 큐. poll_read
    outgoing_blocks: VecDeque<Secs2Variant>,
    /// 외부로 타임아웃 시작 알리는 큐. poll_timeout
    outgoing_timeout_queue: VecDeque<TimeoutTicket>,
    /// 외부 발송할 이벤트 목록을 저장해두는 큐. poll_event
    event_queue: VecDeque<Secs1MessageProtocolEvent>,

    timeout_manager: TimeoutManager,
    /** 내부 상태 값 */
    /// SECS-I message protocol current state
    state: Secs1MessageState,
    /// 중복 수신 방지용 헤더 정보
    last_header: Option<Secs1BlockHeader>,

    /// 통신 중인 디바이스의 ID
    device_id: DeviceId,
}

impl Secs1MessageProtocolMachine {
    pub fn new(config: &Secs1TransportConfig) -> Self {
        Self {
            incoming_buffer: VecDeque::new(),
            outgoing_buffer: VecDeque::new(),
            incoming_blocks: VecDeque::new(),
            outgoing_blocks: VecDeque::new(),
            outgoing_timeout_queue: VecDeque::new(),
            event_queue: VecDeque::new(),
            timeout_manager: TimeoutManager::new(),
            state: Secs1MessageState::Idle,
            last_header: None,
            device_id: config.device_id.clone(),
        }
    }

    fn run(&mut self) {

    }

        /// device id가 알려진 장치인지 체크
    fn is_known_device(&self, device_id: &DeviceId) -> bool {
        self.device_id == *device_id
    }

    /// 헤더가 이전에 수신한 헤더와 동일한지 체크 (중복 수신 방지)
    fn is_duplicate(&self, header: &Secs1BlockHeader) -> bool {
        if let Some(last) = &self.last_header {
            return last == header;
        }
        return false;
    }

    /// 헤더가 현재 기대하는 헤더인지 체크
    fn is_expected(&self, header: &Secs1BlockHeader) -> bool {
        match &self.state {
            Secs1MessageState::Receiving {
                expected_header, ..
            } => expected_header == header,
            Secs1MessageState::WaitingForReply { expected_header } => expected_header == header,
            _ => false,
        }
    }

    fn is_primary_first_block(&self, header: &Secs1BlockHeader) -> bool {
        header.is_primary() && header.is_first_block()
    }

    fn reset_to_idle(&mut self) {
        self.timeout_manager.cancel(SecsTimeoutUnit::T3);
        self.state = Secs1MessageState::Idle;
    }
}

impl Protocol<Secs1Block, Secs2Variant, Secs1MessageProtocolEvent> for Secs1MessageProtocolMachine {
    type Rout = Secs2Variant;
    type Wout = Secs1Block;
    type Eout = Secs1MessageProtocolEvent;
    type Error = SecsTransportError;
    type Time = TimeoutTicket;

    fn handle_read(&mut self, msg: Secs1Block) -> Result<(), Self::Error> {
        self.incoming_buffer.push_back(msg);
        // self.run();
        Ok(())
    }

    fn poll_read(&mut self) -> Option<Self::Rout> {
        self.outgoing_blocks.pop_front()
    }

    fn handle_write(&mut self, msg: Secs2Variant) -> Result<(), Self::Error> {
        self.incoming_blocks.push_back(msg);
        Ok(())
    }

    fn poll_write(&mut self) -> Option<Self::Wout> {
        self.outgoing_buffer.pop_front()
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
        self.outgoing_timeout_queue.pop_front()
    }

    fn handle_event(&mut self, _evt: Secs1MessageProtocolEvent) -> Result<(), Self::Error> {
        todo!()
        // 외부에서 받을 데이터가 있는지 고민 중.
        // timeout 개념이 존재해서 "즉시" 초기화 필요한 경우가 드물다.
    }

    /// 외부 시스템에서 전송할 메시지를 가져간다.
    fn poll_event(&mut self) -> Option<Self::Eout> {
        self.event_queue.pop_front()
    }
}

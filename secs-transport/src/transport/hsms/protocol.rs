use crate::transport::hsms::config::HsmsTransportConfig;

pub mod assembler;
pub mod connection;

use alloc::collections::BTreeSet;
use alloc::collections::VecDeque;
use alloc::vec::Vec;

use sansio::Protocol;
use secs_ii::FunctionId;
use secs_ii::StreamId;

use crate::transport::ConnectionRole;
use crate::transport::SecsTimeoutUnit;
use crate::transport::SessionId;
use crate::transport::SystemByte;
use crate::transport::TimeoutTicket;
use crate::transport::error::SecsTransportError;
use crate::transport::hsms::HsmsHeader;
use crate::transport::hsms::HsmsMessage;
use crate::transport::hsms::protocol::assembler::HsmsAssembler;
use crate::util::time::TimeoutManager;

/// 트랜잭션 요청 - 응답 시 timeout 처리를 위한 매핑 정보
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct MessageInfo(pub StreamId, pub FunctionId, pub SystemByte);

/// HSMS 전송 계층 대응
pub struct HsmsMessageMachine {
    /// byte to message 조립기
    msg_assembler: HsmsAssembler,
    /// serial 송신할 데이터를 임시로 보관하는 큐. poll_write
    outgoing_buffer: VecDeque<u8>,
    /// hsms 통신을 통해 수신한 블록을 저장하는 큐. poll_read
    outgoing_msgs: VecDeque<HsmsMessage>,
    /// 외부로 타임아웃 시작 알리는 큐. poll_timeout
    outgoing_timeouts: VecDeque<TimeoutTicket>,
    /// 외부로 이벤트 알리는 큐. poll_events
    outgoing_events: VecDeque<HsmsMessageEvent>,
    /// timeout 처리 객체
    timeout_manager: TimeoutManager,
    /// 현재 hsms 통신에 대한 세션 ID
    session_id: SessionId,
    /// ACTIVE / PASSIVE mode
    connection_mode: ConnectionRole,
    /// reply 메시지에 대한 transaction ID - key mapping
    reply_set: BTreeSet<MessageInfo>,
}

/// 외부로부터 받는 신호
pub enum HsmsMessageSignal {
    /// 메시지 전송 성공
    SendSuccess(HsmsHeader),
    /// 메시지 전송 실패
    SendFailed(HsmsHeader),
}

/// hsms message 처리 중 발생하는 이벤트
pub enum HsmsMessageEvent {
    SendComplete(MessageInfo),
    RecvComplete(MessageInfo),
    MessageTimeout(MessageInfo, SecsTimeoutUnit),
    /// 비정상적인 상태 전이 등 에러가 발생한 경우
    ErrorOccured(SecsTransportError),
}

impl HsmsMessageMachine {
    pub fn new(config: &HsmsTransportConfig) -> Self {
        Self {
            msg_assembler: HsmsAssembler::new(),
            outgoing_buffer: VecDeque::new(),
            outgoing_msgs: VecDeque::new(),
            outgoing_timeouts: VecDeque::new(),
            outgoing_events: VecDeque::new(),
            timeout_manager: TimeoutManager::new(),
            session_id: config.session_id,
            connection_mode: config.connection_mode,
            reply_set: BTreeSet::new(),
        }
    }

    /// 외부 시스템에 전송할 메시지를 담는다.
    fn emit_event(&mut self, output: HsmsMessageEvent) {
        self.outgoing_events.push_back(output);
    }

    /// timeout을 시작한다.
    fn start_timeout(&mut self, timeout: SecsTimeoutUnit) {
        log::debug!("[message] start timeout {:?}", timeout);
        let ticket = self.timeout_manager.issue(timeout);
        self.outgoing_timeouts.push_back(ticket);
    }

    /// timeout을 취소한다.
    fn cancel_timeout(&mut self, timeout: SecsTimeoutUnit) {
        log::debug!("[message] cancel timeout {:?}", timeout);
        self.timeout_manager.cancel(timeout);
    }

    fn process_control_msg(&mut self, msg: HsmsMessage) {}
}

impl Protocol<&[u8], HsmsMessage, HsmsMessageSignal> for HsmsMessageMachine {
    type Rout = HsmsMessage;
    type Wout = Vec<u8>;
    type Eout = HsmsMessageEvent;
    type Error = SecsTransportError;
    type Time = TimeoutTicket;

    fn handle_read(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        // 데이터가 들어왔으므로 t8 timeout 걸기
        self.cancel_timeout(SecsTimeoutUnit::T8);

        self.msg_assembler.handle_read(bytes)?;

        // 생성된 메시지 외부로 전달
        while let Some(msg) = self.msg_assembler.poll_message() {
            if msg.is_control() {
                self.process_control_msg(msg);
            } else {
                self.outgoing_msgs.push_back(msg);
            }
        }

        // 버퍼에 남은 내용이 있는 경우 timeout 다시 시작
        if self.msg_assembler.has_remained() {
            self.start_timeout(SecsTimeoutUnit::T8);
        }

        Ok(())
    }

    fn poll_read(&mut self) -> Option<Self::Rout> {
        self.outgoing_msgs.pop_front()
    }

    fn handle_write(&mut self, msg: HsmsMessage) -> Result<(), Self::Error> {
        match msg.to_bytes() {
            Ok(bytes) => {
                self.outgoing_buffer.extend(bytes);
            }
            Err(error) => {
                self.emit_event(HsmsMessageEvent::ErrorOccured(error));
            }
        }

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

        match expired_unit {
            SecsTimeoutUnit::T3(transaction_key) => todo!(),
            SecsTimeoutUnit::T5 => todo!(),
            SecsTimeoutUnit::T6 => todo!(),
            SecsTimeoutUnit::T7 => todo!(),
            SecsTimeoutUnit::T8 => todo!(),
            _ => {}
        }

        todo!("구체적인 타임아웃 처리");

        Ok(())
    }

    fn poll_timeout(&mut self) -> Option<Self::Time> {
        self.outgoing_timeouts.pop_front()
    }

    fn handle_event(&mut self, event: HsmsMessageSignal) -> Result<(), Self::Error> {
        todo!("signal 처리");

        Ok(())
    }

    /// 외부 시스템에서 전송할 메시지를 가져간다.
    fn poll_event(&mut self) -> Option<Self::Eout> {
        self.outgoing_events.pop_front()
    }
}

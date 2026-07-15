pub mod config;
pub mod connection;
pub mod frame;

use num_enum::{IntoPrimitive, TryFromPrimitive};
use secs_ii::{FunctionId, Secs2Message, StreamId};

use crate::transport::{SessionId, SystemByte};

/// HSMS 공통 헤더
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HsmsHeader {
    pub session_id: SessionId,
    pub byte2: u8,
    pub byte3: u8,
    pub ptype: HsmsPType,
    pub stype: HsmsSType,
    pub system_byte: SystemByte,
}

impl HsmsHeader {
    pub fn new(
        session_id: SessionId,
        byte2: u8,
        byte3: u8,
        ptype: HsmsPType,
        stype: HsmsSType,
        system_byte: SystemByte,
    ) -> Self {
        Self {
            session_id,
            byte2,
            byte3,
            ptype,
            stype,
            system_byte,
        }
    }

    pub fn is_data(&self) -> bool {
        self.stype.is_data()
    }

    pub fn is_control(&self) -> bool {
        self.stype.is_control()
    }

    pub fn stream(&self) -> StreamId {
        StreamId(self.byte2)
    }

    pub fn function(&self) -> FunctionId {
        FunctionId(self.byte3 & 0x7f)
    }

    pub fn need_reply(&self) -> bool {
        self.byte3 & 0x80 != 0
    }

    pub fn with_stream_function_need_reply(
        session_id: SessionId,
        stream: StreamId,
        function: FunctionId,
        need_reply: bool,
        stype: HsmsSType,
        system_byte: SystemByte,
    ) -> Self {
        let byte2 = stream.0;
        let byte3 = function.0 | if need_reply { 0x80 } else { 0x00 };
        Self::new(session_id, byte2, byte3, HsmsPType::SECS2, stype, system_byte)
    }
}

/// HSMS Data Message
pub struct HsmsDataMessage {
    pub header: HsmsHeader,
    pub payload: Secs2Message,
}

impl HsmsDataMessage {
    pub fn new(header: HsmsHeader, payload: Secs2Message) -> Self {
        debug_assert!(header.is_data());
        Self { header, payload }
    }
}

/// HSMS Control Message
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HsmsControlMessage {
    pub header: HsmsHeader,
}

impl HsmsControlMessage {
    pub fn new(header: HsmsHeader) -> Self {
        debug_assert!(header.is_control());
        Self { header }
    }
}

/// HSMS 메시지
pub enum HsmsMessage {
    Data(HsmsDataMessage),
    Control(HsmsControlMessage),
}

impl HsmsMessage {
    pub fn header(&self) -> &HsmsHeader {
        match self {
            Self::Data(msg) => &msg.header,
            Self::Control(msg) => &msg.header,
        }
    }

    pub fn into_header(self) -> HsmsHeader {
        match self {
            Self::Data(msg) => msg.header,
            Self::Control(msg) => msg.header,
        }
    }

    pub fn is_data(&self) -> bool {
        self.header().is_data()
    }

    pub fn is_control(&self) -> bool {
        self.header().is_control()
    }
}

/// HSMS Presentation Type -> 데이터 부 인코딩 방식
/// 명세 상 현재 SECS-II 밖에 없음
#[derive(Debug, TryFromPrimitive, IntoPrimitive, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum HsmsPType {
    /// SECS-II
    SECS2 = 0,
}

impl HsmsPType {
    pub fn is_secs2(self) -> bool {
        matches!(self, Self::SECS2)
    }
}

/// HSMS Session Type -> 메시지 타입
#[derive(Debug, TryFromPrimitive, IntoPrimitive, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum HsmsSType {
    DataMessage = 0,
    SelectRequest = 1,
    SelectResponse = 2,
    DeselectRequest = 3,
    DeselectResponse = 4,
    LinktestRequest = 5,
    LinktestResponse = 6,
    RejectRequest = 7,
    SeparateRequest = 9,
}

impl HsmsSType {
    pub fn is_data(self) -> bool {
        matches!(self, Self::DataMessage)
    }

    pub fn is_control(self) -> bool {
        !self.is_data()
    }
}

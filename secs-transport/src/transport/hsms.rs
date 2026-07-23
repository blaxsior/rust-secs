pub mod config;
pub mod protocol;

use alloc::vec::Vec;
use core::convert::TryFrom;

use num_enum::{IntoPrimitive, TryFromPrimitive};
use secs_ii::{FunctionId, StreamId, item::Secs2Variant};

use crate::transport::error::SecsTransportError;
use crate::transport::{SessionId, SystemByte};

const WITHOUT_MSB: u8 = 0x7F;
const MSB_ONLY: u8 = 0x80;

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

    pub fn data(
        session_id: SessionId,
        stream: StreamId,
        function: FunctionId,
        need_reply: bool,
        system_byte: SystemByte,
    ) -> Self {
        let stream = stream.0 & WITHOUT_MSB;
        let byte2 = if need_reply {
            stream | MSB_ONLY
        } else {
            stream
        };
        let byte3 = function.0;

        Self::new(
            session_id,
            byte2,
            byte3,
            HsmsPType::SECS2,
            HsmsSType::DataMessage,
            system_byte,
        )
    }

    /// control 메시지 생성자
    pub fn control(byte2: u8, byte3: u8, stype: HsmsSType, system_byte: SystemByte) -> Self {
        Self::new(
            SessionId(0xFFFF),
            byte2,
            byte3,
            HsmsPType::SECS2,
            stype,
            system_byte,
        )
    }

    pub fn is_data(&self) -> bool {
        self.stype.is_data()
    }

    pub fn is_control(&self) -> bool {
        self.stype.is_control()
    }

    pub fn stream(&self) -> StreamId {
        StreamId(self.byte2 & WITHOUT_MSB)
    }

    pub fn function(&self) -> FunctionId {
        FunctionId(self.byte3)
    }

    /// control 성공인지 여부. byte3 = 0으로 체크 가능
    pub fn is_control_success(&self) -> bool {
        self.is_control() && self.byte3 == 0
    }

    pub fn need_reply(&self) -> bool {
        self.byte2 & MSB_ONLY != 0
    }

    pub fn to_bytes(&self) -> [u8; 10] {
        let mut h = [0u8; 10];

        h[0..2].copy_from_slice(&self.session_id.0.to_be_bytes());
        h[2] = self.byte2;
        h[3] = self.byte3;
        h[4] = self.ptype.into();
        h[5] = self.stype.into();
        h[6..10].copy_from_slice(&self.system_byte.0.to_be_bytes());

        h
    }
}

impl TryFrom<[u8; 10]> for HsmsHeader {
    type Error = SecsTransportError;

    fn try_from(h: [u8; 10]) -> Result<Self, Self::Error> {
        Ok(Self {
            session_id: SessionId(u16::from_be_bytes([h[0], h[1]])),
            byte2: h[2],
            byte3: h[3],
            ptype: HsmsPType::try_from(h[4]).map_err(|_| SecsTransportError::InvalidBlockHeader)?,
            stype: HsmsSType::try_from(h[5]).map_err(|_| SecsTransportError::InvalidBlockHeader)?,
            system_byte: SystemByte(u32::from_be_bytes([h[6], h[7], h[8], h[9]])),
        })
    }
}

/// HSMS Message
#[derive(Debug)]
pub struct HsmsMessage {
    pub header: HsmsHeader,
    pub payload: Option<Secs2Variant>,
}

impl HsmsMessage {
    pub fn new(header: HsmsHeader, payload: Option<Secs2Variant>) -> Self {
        Self { header, payload }
    }

    pub fn header(&self) -> &HsmsHeader {
        &self.header
    }

    pub fn payload(&self) -> Option<&Secs2Variant> {
        self.payload.as_ref()
    }

    pub fn is_data(&self) -> bool {
        self.header().is_data()
    }

    pub fn is_control(&self) -> bool {
        self.header().is_control()
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, SecsTransportError> {
        let mut bytes = Vec::new();
        let header_bytes = self.header().to_bytes();

        let mut message_text = Vec::new();
        if let Some(payload) = self.payload() {
            use secs_ii::convert::secs2::serialize::Encode;
            payload
                .encode(&mut message_text)
                .map_err(|_| SecsTransportError::InvalidBlockHeader)?;
        }

        let len = (header_bytes.len() + message_text.len()) as u32;
        bytes.extend_from_slice(&len.to_be_bytes());
        bytes.extend_from_slice(&header_bytes);
        bytes.extend_from_slice(&message_text);
        Ok(bytes)
    }
}

impl TryFrom<&[u8]> for HsmsMessage {
    type Error = SecsTransportError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 14 {
            return Err(SecsTransportError::InvalidBlockLength(value.len()));
        }

        let len = u32::from_be_bytes(value[0..4].try_into().unwrap()) as usize;
        if value.len() < 4 + len {
            return Err(SecsTransportError::InvalidBlockLength(value.len()));
        }

        let header_bytes: [u8; 10] = value[4..14]
            .try_into()
            .map_err(|_| SecsTransportError::InvalidBlockHeader)?;
        let header = HsmsHeader::try_from(header_bytes)?;
        if header.ptype != HsmsPType::SECS2 {
            return Err(SecsTransportError::InvalidBlockHeader);
        }

        let payload = if header.is_data() {
            let body_bytes = &value[14..4 + len];
            Some(
                Secs2Variant::try_from(body_bytes)
                    .map_err(|_| SecsTransportError::InvalidBlockHeader)?,
            )
        } else {
            None
        };

        Ok(HsmsMessage::new(header, payload))
    }
}

/// HSMS Presentation Type
#[derive(Debug, TryFromPrimitive, IntoPrimitive, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum HsmsPType {
    /// SECS-II
    SECS2 = 0,
}

/// HSMS Session Type
#[derive(Debug, TryFromPrimitive, IntoPrimitive, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum HsmsSType {
    DataMessage = 0,
    SelectReq = 1,
    SelectRsp = 2,
    DeselectReq = 3,
    DeselectRsp = 4,
    LinktestReq = 5,
    LinktestRsp = 6,
    RejectReq = 7,
    SeparateReq = 9,
}

impl HsmsSType {
    pub fn is_data(self) -> bool {
        matches!(self, Self::DataMessage)
    }

    pub fn is_control(self) -> bool {
        !self.is_data()
    }
}

/// HSMS select 상태 정보
#[derive(Debug, TryFromPrimitive, IntoPrimitive, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum HsmsSelectStatus {
    /// select 성공
    Success = 0,
    /// 이미 select 상태
    AlreadyActive = 1,
    /// select  준비가 안됨
    NotReady = 2,
    /// TCP 연결 성공, 세션 연결 실패(이미 다른 엔티티와 연결된 상태)
    ConnectionExhaust = 3,
}

/// HSMS deselect 상태 정보
#[derive(Debug, TryFromPrimitive, IntoPrimitive, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum HsmsDeselectStatus {
    /// deselect 성공
    CommunicationEnd = 0,
    /// 아직 select 되지 않음
    NotEstablished = 1,
    /// 아직 communication 진행 중(당장 끊어야 한다면 seperate)
    Busy = 2,
}

/// HSMS reject reason code
#[derive(Debug, TryFromPrimitive, IntoPrimitive, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum HsmsRejectReasonCode {
    NotSupportedSType = 1,
    NotSupportedPType = 2,
    TransactionNotOpen = 3,
    NotSelected = 4,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_header_with_reply() {
        let header = HsmsHeader::data(
            SessionId(10),
            StreamId(1),
            FunctionId(3),
            true,
            SystemByte(100),
        );

        assert!(header.is_data());
        assert!(!header.is_control());
        assert_eq!(header.session_id, SessionId(10));
        assert_eq!(header.stream(), StreamId(1));
        assert_eq!(header.function(), FunctionId(3));
        assert!(header.need_reply());
        assert_eq!(header.byte2, 0x81);
        assert_eq!(header.byte3, 0x03);
        assert_eq!(header.ptype, HsmsPType::SECS2);
        assert_eq!(header.stype, HsmsSType::DataMessage);
        assert_eq!(header.system_byte, SystemByte(100));
    }

    #[test]
    fn test_data_header_without_reply() {
        let header = HsmsHeader::data(
            SessionId(10),
            StreamId(1),
            FunctionId(4),
            false,
            SystemByte(101),
        );

        assert!(header.is_data());
        assert_eq!(header.stream(), StreamId(1));
        assert_eq!(header.function(), FunctionId(4));
        assert!(!header.need_reply());
        assert_eq!(header.byte2, 0x01);
        assert_eq!(header.byte3, 0x04);
    }
}

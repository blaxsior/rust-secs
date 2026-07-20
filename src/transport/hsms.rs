pub mod config;
pub mod connection;
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
pub enum HsmsMessage {
    Message {
        header: HsmsHeader,
        payload: Option<Secs2Variant>,
    },
}

impl HsmsMessage {
    pub fn new(header: HsmsHeader, payload: Option<Secs2Variant>) -> Self {
        Self::Message { header, payload }
    }

    pub fn header(&self) -> &HsmsHeader {
        match self {
            Self::Message { header, .. } => header,
        }
    }

    pub fn payload(&self) -> Option<&Secs2Variant> {
        match self {
            Self::Message { payload, .. } => payload.as_ref(),
        }
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

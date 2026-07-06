use alloc::vec::Vec;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use secs_ii::{FunctionId, StreamId};

use crate::transport::{DeviceId, SystemByte, error::SecsTransportError};

const WITHOUT_MSB: u8 = 0x7F;
const MSB_ONLY: u8 = 0x80;

///
/// SECS-I Block Transfer Protocol 중 사용되는 구조체
///
#[derive(Debug, PartialEq, Eq)]
pub struct Secs1Block {
    pub header: Secs1BlockHeader,
    pub data: Vec<u8>,
}

impl Secs1Block {
    // pub fn new(header: Secs1BlockHeader, )

    pub fn checksum(&self) -> u16 {
        self.header
            .to_bytes()
            .iter()
            .chain(self.data.iter())
            .fold(0u16, |acc, b| acc.wrapping_add(*b as u16))
    }

    pub fn verify_checksum(&self, expected: u16) -> bool {
        self.checksum() == expected
    }

    /// bytes 배열로 변환
    pub fn to_bytes(&self) -> Vec<u8> {
        let header = self.header.to_bytes();

        let mut buf = Vec::with_capacity(header.len() + self.data.len());

        buf.extend_from_slice(&header);
        buf.extend_from_slice(&self.data);

        buf
    }
}

impl TryFrom<&[u8]> for Secs1Block {
    type Error = SecsTransportError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 10 || value.len() > 254 {
            return Err(SecsTransportError::InvalidBlock);
        }

        let raw_header: [u8; 10] = value[0..10]
            .try_into()
            .map_err(|_| SecsTransportError::InvalidBlock)?;

        let header = Secs1BlockHeader::try_from(raw_header)?;

        Ok(Self {
            header,
            data: value[10..].to_vec(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rbit(bool);
impl Rbit {
    pub const FORWARD: Self = Self(false);
    pub const REVERSE: Self = Self(true);

    /// 보수 값을 반환
    pub const fn complement(self) -> Self {
        Self(!self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageDirection {
    /// Host -> Equipment (R = 0)
    Forward,
    /// Equipment -> Host (R = 1)
    Reverse,
}

impl From<Rbit> for MessageDirection {
    fn from(value: Rbit) -> Self {
        if value.0 {
            Self::Reverse
        } else {
            Self::Forward
        }
    }
}

impl From<MessageDirection> for Rbit {
    fn from(value: MessageDirection) -> Self {
        match value {
            MessageDirection::Forward => Rbit(false),
            MessageDirection::Reverse => Rbit(true),
        }
    }
}
///
/// SECS-I block header을 표현하는 구조체
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Secs1BlockHeader {
    /// reverse bit. eqp -> host인 경우 true
    pub rbit: Rbit,
    /// 통신 대상 장치의 ID 값
    pub device_id: DeviceId,

    /// wait bit. primary msg에 대한 응답이 필요한 경우 true
    pub wbit: bool,
    pub stream: StreamId,
    pub function: FunctionId,

    /// end bit. 마지막 block인 경우 true
    pub ebit: bool,
    /// block 번호. 단일 block은 0 허용, 아니면 1부터 시작하여 1씩 증가
    pub block_no: u16,
    /// block transfer에 대한 트랜잭션을 식별하기 위한 byte 정보
    pub system_bytes: SystemByte,
}

impl Secs1BlockHeader {
    pub fn to_bytes(&self) -> [u8; 10] {
        let mut h = [0u8; 10];

        h[0] = ((self.rbit.0 as u8) << 7) | ((self.device_id.0 >> 8) as u8 & WITHOUT_MSB);
        h[1] = self.device_id.0 as u8;

        h[2] = ((self.wbit as u8) << 7) | (self.stream.0 & WITHOUT_MSB);
        h[3] = self.function.0;

        h[4] = ((self.ebit as u8) << 7) | ((self.block_no >> 8) as u8 & WITHOUT_MSB);
        h[5] = self.block_no as u8;

        h[6..10].copy_from_slice(&self.system_bytes.0.to_be_bytes());

        h
    }

    /// 마지막 block인지 여부
    pub fn is_end(&self) -> bool {
        self.ebit
    }

    /// 응답을 요구하는지 여부
    pub fn need_reply(&self) -> bool {
        self.wbit
    }

    pub fn direction(&self) -> MessageDirection {
        self.rbit.into()
    }

    /// primary message인지 여부
    pub fn is_primary(&self) -> bool {
        self.function.is_primary()
    }

    /// primary message인지 여부
    pub fn is_secondary(&self) -> bool {
        self.function.is_secondary()
    }

    /// 첫번째 block인지 여부
    pub fn is_first_block(&self) -> bool {
        self.block_no == 1 || (self.block_no == 0 && self.ebit)
    }
}

impl TryFrom<[u8; 10]> for Secs1BlockHeader {
    type Error = SecsTransportError;

    fn try_from(h: [u8; 10]) -> Result<Self, Self::Error> {
        Ok(Self {
            rbit: Rbit(h[0] & MSB_ONLY != 0),
            device_id: DeviceId(u16::from_be_bytes([h[0] & WITHOUT_MSB, h[1]])),

            wbit: h[2] & MSB_ONLY != 0,
            stream: StreamId(h[2] & WITHOUT_MSB),
            function: FunctionId(h[3]),

            ebit: h[4] & MSB_ONLY != 0,
            block_no: u16::from_be_bytes([h[4] & WITHOUT_MSB, h[5]]),

            system_bytes: SystemByte(u32::from_be_bytes([h[6], h[7], h[8], h[9]])),
        })
    }
}

/// Secs-I 통신 Block Transfer Protocol에서 사용되는 코드
#[derive(Debug, TryFromPrimitive, IntoPrimitive, PartialEq, Eq)]
#[repr(u8)]
pub enum Secs1HandshakeCode {
    /// request to send
    ENQ = 0b00000101,
    /// ready to receive
    EOT = 0b00000100,
    /// correct reception
    ACK = 0b00000110,
    // incorrect reception
    NAK = 0b00010101,
}

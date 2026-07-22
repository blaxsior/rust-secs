use alloc::vec::Vec;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use secs_ii::{FunctionId, StreamId};

use crate::transport::{DeviceId, Rbit, SystemByte, Wbit, error::SecsTransportError};

const WITHOUT_MSB: u8 = 0x7F;
const MSB_ONLY: u8 = 0x80;
const HEADER_LEN: usize = 10;
const CHECKSUM_LEN: usize = 2;
const LENGTH_LEN: usize = 1;
const MIN_BLOCK_BODY_LEN: usize = HEADER_LEN;
const MAX_BLOCK_BODY_LEN: usize = 254;

/// SECS-I Block Transfer Protocol에서 사용하는 Block.
///
/// Wire format은 `length + header + data + checksum`이다.
#[derive(Debug, PartialEq, Eq)]
pub struct Secs1Block {
    pub header: Secs1BlockHeader,
    pub data: Vec<u8>,
}

impl Secs1Block {
    pub fn length(&self) -> u8 {
        let length = HEADER_LEN + self.data.len();
        assert!(
            length <= MAX_BLOCK_BODY_LEN,
            "SECS-I block body length must be <= 254"
        );
        length as u8
    }

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

    fn to_body_bytes(&self) -> Vec<u8> {
        let header = self.header.to_bytes();
        let mut buf = Vec::with_capacity(header.len() + self.data.len());

        buf.extend_from_slice(&header);
        buf.extend_from_slice(&self.data);

        buf
    }

    /// `length + header + data + checksum` 형태의 SECS-I block bytes로 변환한다.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.length() as usize + LENGTH_LEN + CHECKSUM_LEN);

        bytes.push(self.length());
        bytes.extend_from_slice(&self.to_body_bytes());
        bytes.extend_from_slice(&self.checksum().to_be_bytes());

        bytes
    }
}

impl TryFrom<&[u8]> for Secs1Block {
    type Error = SecsTransportError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let Some(length) = value.first().copied() else {
            return Err(SecsTransportError::InvalidBlockLength(0));
        };

        let body_len = length as usize;
        if body_len < MIN_BLOCK_BODY_LEN || body_len > MAX_BLOCK_BODY_LEN {
            return Err(SecsTransportError::InvalidBlockLength(body_len));
        }

        let expected_wire_len = LENGTH_LEN + body_len + CHECKSUM_LEN;
        if value.len() != expected_wire_len {
            return Err(SecsTransportError::InvalidBlockLength(value.len()));
        }

        let body_start = LENGTH_LEN;
        let body_end = body_start + body_len;
        let checksum_start = body_end;

        let raw_header: [u8; HEADER_LEN] = value[body_start..body_start + HEADER_LEN]
            .try_into()
            .map_err(|_| SecsTransportError::InvalidBlockHeader)?;

        let header = Secs1BlockHeader::try_from(raw_header)?;
        let data = value[body_start + HEADER_LEN..body_end].to_vec();
        let expected_checksum =
            u16::from_be_bytes([value[checksum_start], value[checksum_start + 1]]);

        let block = Self { header, data };
        if !block.verify_checksum(expected_checksum) {
            return Err(SecsTransportError::BlockError);
        }

        Ok(block)
    }
}

/// SECS-I block header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Secs1BlockHeader {
    /// 통신 대상 장치 ID.
    pub device_id: DeviceId,
    /// Reverse bit.
    pub rbit: Rbit,
    /// Wait bit. Primary message에 대한 응답이 필요한 경우 true.
    pub wbit: Wbit,
    pub stream: StreamId,
    pub function: FunctionId,
    /// End bit. 마지막 block인 경우 true.
    pub ebit: bool,
    /// Block 번호. 단일 block은 0 허용, 다중 block은 1부터 증가.
    pub block_no: u16,
    /// Transaction 식별을 위한 system byte.
    pub system_byte: SystemByte,
}

impl Secs1BlockHeader {
    pub fn to_bytes(&self) -> [u8; HEADER_LEN] {
        let mut h = [0u8; HEADER_LEN];

        h[0] = ((self.rbit.0 as u8) << 7) | ((self.device_id.0 >> 8) as u8 & WITHOUT_MSB);
        h[1] = self.device_id.0 as u8;

        h[2] = ((self.wbit.0 as u8) << 7) | (self.stream.0 & WITHOUT_MSB);
        h[3] = self.function.0;

        h[4] = ((self.ebit as u8) << 7) | ((self.block_no >> 8) as u8 & WITHOUT_MSB);
        h[5] = self.block_no as u8;

        h[6..10].copy_from_slice(&self.system_byte.0.to_be_bytes());

        h
    }

    pub fn is_end(&self) -> bool {
        self.ebit
    }

    pub fn need_reply(&self) -> bool {
        self.wbit.need_reply()
    }

    pub fn is_primary(&self) -> bool {
        self.function.is_primary()
    }

    pub fn is_secondary(&self) -> bool {
        self.function.is_secondary()
    }

    pub fn is_first_block(&self) -> bool {
        self.block_no == 1 || (self.block_no == 0 && self.ebit)
    }
}

impl TryFrom<[u8; HEADER_LEN]> for Secs1BlockHeader {
    type Error = SecsTransportError;

    fn try_from(h: [u8; HEADER_LEN]) -> Result<Self, Self::Error> {
        Ok(Self {
            rbit: Rbit(h[0] & MSB_ONLY != 0),
            device_id: DeviceId(u16::from_be_bytes([h[0] & WITHOUT_MSB, h[1]])),

            wbit: Wbit(h[2] & MSB_ONLY != 0),
            stream: StreamId(h[2] & WITHOUT_MSB),
            function: FunctionId(h[3]),

            ebit: h[4] & MSB_ONLY != 0,
            block_no: u16::from_be_bytes([h[4] & WITHOUT_MSB, h[5]]),

            system_byte: SystemByte(u32::from_be_bytes([h[6], h[7], h[8], h[9]])),
        })
    }
}

/// SECS-I Block Transfer Protocol에서 사용하는 handshake code.
#[derive(Debug, TryFromPrimitive, IntoPrimitive, PartialEq, Eq)]
#[repr(u8)]
pub enum Secs1HandshakeCode {
    /// Request to send.
    ENQ = 0b00000101,
    /// Ready to receive.
    EOT = 0b00000100,
    /// Correct reception.
    ACK = 0b00000110,
    /// Incorrect reception.
    NAK = 0b00010101,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secs1_block_header_to_bytes() {
        let header = Secs1BlockHeader {
            rbit: Rbit(true),
            device_id: DeviceId(0x1234),
            wbit: Wbit(true),
            stream: StreamId(0x45),
            function: FunctionId(0x67),
            ebit: true,
            block_no: 0x1234,
            system_byte: SystemByte(0x89ABCDEF),
        };

        assert_eq!(
            header.to_bytes(),
            [0x92, 0x34, 0xC5, 0x67, 0x92, 0x34, 0x89, 0xAB, 0xCD, 0xEF,]
        );
    }

    #[test]
    fn test_secs1_block_header_try_from() {
        let bytes = [0x92, 0x34, 0xC5, 0x67, 0x92, 0x34, 0x89, 0xAB, 0xCD, 0xEF];

        let header = Secs1BlockHeader::try_from(bytes).unwrap();

        assert_eq!(header.rbit, Rbit(true));
        assert_eq!(header.device_id, DeviceId(0x1234));
        assert_eq!(header.wbit, Wbit(true));
        assert_eq!(header.stream, StreamId(0x45));
        assert_eq!(header.function, FunctionId(0x67));
        assert!(header.ebit);
        assert_eq!(header.block_no, 0x1234);
        assert_eq!(header.system_byte, SystemByte(0x89ABCDEF));
    }

    #[test]
    fn test_secs1_block_header_round_trip() {
        let header = Secs1BlockHeader {
            rbit: Rbit(true),
            device_id: DeviceId(0x1234),
            wbit: Wbit(true),
            stream: StreamId(0x45),
            function: FunctionId(0x67),
            ebit: true,
            block_no: 0x1234,
            system_byte: SystemByte(0x89ABCDEF),
        };

        let bytes = header.to_bytes();
        let decoded = Secs1BlockHeader::try_from(bytes).unwrap();

        assert_eq!(decoded, header);
    }

    fn sample_header() -> Secs1BlockHeader {
        Secs1BlockHeader {
            rbit: Rbit(true),
            device_id: DeviceId(0x1234),
            wbit: Wbit(true),
            stream: StreamId(0x45),
            function: FunctionId(0x67),
            ebit: true,
            block_no: 0x1234,
            system_byte: SystemByte(0x89ABCDEF),
        }
    }

    fn sample_block() -> Secs1Block {
        Secs1Block {
            header: sample_header(),
            data: vec![0x11, 0x22, 0x33, 0x44],
        }
    }

    fn sample_block_bytes() -> [u8; 17] {
        [
            // length
            0x0E, // body
            0x92, 0x34, 0xC5, 0x67, 0x92, 0x34, 0x89, 0xAB, 0xCD, 0xEF, 0x11, 0x22, 0x33, 0x44,
            // checksum
            0x06, 0x52,
        ]
    }

    #[test]
    fn test_secs1_block_to_bytes() {
        let block = sample_block();

        assert_eq!(block.to_bytes(), sample_block_bytes());
    }

    #[test]
    fn test_secs1_block_try_from() {
        let raw = sample_block_bytes();

        let block = Secs1Block::try_from(raw.as_slice()).unwrap();

        assert_eq!(block.length(), raw[0]);
        assert_eq!(block.header, sample_header());
        assert_eq!(block.data, vec![0x11, 0x22, 0x33, 0x44]);
        assert_eq!(block.checksum(), 0x0652);
    }

    #[test]
    fn test_secs1_block_try_from_rejects_body_only_bytes() {
        let raw = sample_block().to_body_bytes();

        assert!(matches!(
            Secs1Block::try_from(raw.as_slice()),
            Err(SecsTransportError::InvalidBlockLength(14))
        ));
    }

    #[test]
    fn test_secs1_block_try_from_rejects_checksum_mismatch() {
        let mut raw = sample_block_bytes();
        let last = raw.len() - 1;
        raw[last] = raw[last].wrapping_add(1);

        assert!(matches!(
            Secs1Block::try_from(raw.as_slice()),
            Err(SecsTransportError::BlockError)
        ));
    }

    #[test]
    fn test_secs1_block_try_from_rejects_length_mismatch() {
        let mut raw = sample_block_bytes();
        raw[0] -= 1;

        assert!(matches!(
            Secs1Block::try_from(raw.as_slice()),
            Err(SecsTransportError::InvalidBlockLength(17))
        ));
    }

    #[test]
    fn test_secs1_block_round_trip() {
        let block = Secs1Block::try_from(sample_block_bytes().as_slice()).unwrap();

        let bytes = block.to_bytes();
        let decoded = Secs1Block::try_from(bytes.as_slice()).unwrap();

        assert_eq!(decoded, block);
        assert_eq!(bytes, sample_block_bytes());
    }

    #[test]
    fn test_secs1_block_checksum() {
        let block = sample_block();

        let expected = block
            .to_body_bytes()
            .iter()
            .fold(0u16, |acc, b| acc.wrapping_add(*b as u16));

        assert_eq!(block.checksum(), expected);
        assert!(block.verify_checksum(expected));
        assert!(!block.verify_checksum(expected.wrapping_add(1)));
    }

    #[test]
    fn test_secs1_block_try_from_invalid_length() {
        assert!(matches!(
            Secs1Block::try_from(&[][..]),
            Err(SecsTransportError::InvalidBlockLength(0))
        ));

        assert!(matches!(
            Secs1Block::try_from(&[0u8; 9][..]),
            Err(SecsTransportError::InvalidBlockLength(0))
        ));

        assert!(matches!(
            Secs1Block::try_from(&[0x0A, 0, 0][..]),
            Err(SecsTransportError::InvalidBlockLength(3))
        ));

        assert!(matches!(
            Secs1Block::try_from(&[255u8; 255][..]),
            Err(SecsTransportError::InvalidBlockLength(255))
        ));
    }
}

use alloc::vec::Vec;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use secs_ii::{FunctionId, StreamId};

use crate::transport::{DeviceId, Rbit, SystemByte, Wbit, error::SecsTransportError};

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

    pub fn to_bytes_with_checksum(&self) -> Vec<u8> {
        let mut bytes = self.to_bytes();
        let checksum = self.checksum();

        bytes.extend_from_slice(checksum.to_be_bytes().as_slice());
        return bytes;
    }
}

impl TryFrom<&[u8]> for Secs1Block {
    type Error = SecsTransportError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 10 || value.len() > 254 {
            return Err(SecsTransportError::InvalidBlockLength(value.len()));
        }

        let raw_header: [u8; 10] = value[0..10]
            .try_into()
            .map_err(|_| SecsTransportError::InvalidBlockHeader)?;

        let header = Secs1BlockHeader::try_from(raw_header)?;

        Ok(Self {
            header,
            data: value[10..].to_vec(),
        })
    }
}

///
/// SECS-I block header을 표현하는 구조체
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Secs1BlockHeader {
    /// 통신 대상 장치의 ID 값
    pub device_id: DeviceId,
    /// reverse bit. eqp -> host인 경우 true
    pub rbit: Rbit,
    /// wait bit. primary msg에 대한 응답이 필요한 경우 true
    pub wbit: Wbit,
    pub stream: StreamId,
    pub function: FunctionId,
    /// end bit. 마지막 block인 경우 true
    pub ebit: bool,
    /// block 번호. 단일 block은 0 허용, 아니면 1부터 시작하여 1씩 증가
    pub block_no: u16,
    /// block transfer에 대한 트랜잭션을 식별하기 위한 byte 정보
    pub system_byte: SystemByte,
}

impl Secs1BlockHeader {
    pub fn to_bytes(&self) -> [u8; 10] {
        let mut h = [0u8; 10];

        h[0] = ((self.rbit.0 as u8) << 7) | ((self.device_id.0 >> 8) as u8 & WITHOUT_MSB);
        h[1] = self.device_id.0 as u8;

        h[2] = ((self.wbit.0 as u8) << 7) | (self.stream.0 & WITHOUT_MSB);
        h[3] = self.function.0;

        h[4] = ((self.ebit as u8) << 7) | ((self.block_no >> 8) as u8 & WITHOUT_MSB);
        h[5] = self.block_no as u8;

        h[6..10].copy_from_slice(&self.system_byte.0.to_be_bytes());

        h
    }

    /// 마지막 block인지 여부
    pub fn is_end(&self) -> bool {
        self.ebit
    }

    /// 응답을 요구하는지 여부
    pub fn need_reply(&self) -> bool {
        self.wbit.need_reply()
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

            wbit: Wbit(h[2] & MSB_ONLY != 0),
            stream: StreamId(h[2] & WITHOUT_MSB),
            function: FunctionId(h[3]),

            ebit: h[4] & MSB_ONLY != 0,
            block_no: u16::from_be_bytes([h[4] & WITHOUT_MSB, h[5]]),

            system_byte: SystemByte(u32::from_be_bytes([h[6], h[7], h[8], h[9]])),
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
            [
                0x92, // R=1, device_id high=0x12
                0x34, 0xC5, // W=1, stream=0x45
                0x67, 0x92, // E=1, block_no high=0x12
                0x34, 0x89, 0xAB, 0xCD, 0xEF,
            ]
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

    #[test]
    fn test_secs1_block_to_bytes() {
        let block = sample_block();

        assert_eq!(
            block.to_bytes(),
            vec![
                0x92, 0x34, 0xC5, 0x67, 0x92, 0x34, 0x89, 0xAB, 0xCD, 0xEF, 0x11, 0x22, 0x33, 0x44,
            ]
        );
    }

    #[test]
    fn test_secs1_block_try_from() {
        let raw = [
            0x92, 0x34, 0xC5, 0x67, 0x92, 0x34, 0x89, 0xAB, 0xCD, 0xEF, 0x11, 0x22, 0x33, 0x44,
        ];

        let block = Secs1Block::try_from(raw.as_slice()).unwrap();

        assert_eq!(block.header, sample_header());
        assert_eq!(block.data, vec![0x11, 0x22, 0x33, 0x44]);
    }

    #[test]
    fn test_secs1_block_round_trip() {
        let block = sample_block();

        let bytes = block.to_bytes();
        let decoded = Secs1Block::try_from(bytes.as_slice()).unwrap();

        assert_eq!(decoded, block);
    }

    #[test]
    fn test_secs1_block_checksum() {
        let block = sample_block();

        let expected = block
            .to_bytes()
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
            Err(SecsTransportError::InvalidBlockLength(9))
        ));

        assert!(matches!(
            Secs1Block::try_from(&[0u8; 255][..]),
            Err(SecsTransportError::InvalidBlockLength(255))
        ));
    }
}

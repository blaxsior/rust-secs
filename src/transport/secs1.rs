pub mod block;
pub mod config;

pub mod convert;
pub mod protocol;

use secs_ii::{FunctionId, StreamId, item::Secs2Variant};

use crate::transport::{DeviceId, Rbit, SystemByte, Wbit, secs1::block::Secs1BlockHeader};

/// SECS-I 통신 중 사용하는 메시지 모델
pub struct Secs1Message {
    pub header: Secs1MessageHeader,
    pub body: Secs2Variant,
}

pub struct Secs1MessageHeader {
    pub device_id: DeviceId,
    pub rbit: Rbit,
    pub wbit: Wbit,
    pub stream: StreamId,
    pub function: FunctionId,
    pub system_byte: SystemByte,
}

impl From<&Secs1BlockHeader> for Secs1MessageHeader {
    fn from(header: &Secs1BlockHeader) -> Self {
        Self {
            device_id: header.device_id,
            rbit: header.rbit,
            wbit: header.wbit,
            stream: header.stream,
            function: header.function,
            system_byte: header.system_byte
        }
    }
}

impl Secs1Message {
    pub fn new(
        header: Secs1MessageHeader,
        body: Secs2Variant,
    ) -> Self {
        Self {
          header,
          body
        }
    }
}

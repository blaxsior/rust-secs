pub mod block;
pub mod config;

pub mod protocol;
pub mod convert;

use secs_ii::Secs2Message;

use crate::transport::{DeviceId, Rbit, SystemByte};

/// SECS-I 통신 중 사용하는 메시지 모델
pub struct Secs1Message {
    pub device_id: DeviceId,
    pub system_byte: SystemByte,
    pub rbit: Rbit,
    pub payload: Secs2Message,
}

impl Secs1Message {
    pub fn new(
        device_id: DeviceId,
        system_byte: SystemByte,
        rbit: Rbit,
        payload: Secs2Message,
    ) -> Self {
        Self {
            device_id,
            system_byte,
            rbit,
            payload,
        }
    }
}
use secs_ii::Secs2Message;

use crate::transport::{DeviceId, Rbit, SystemByte};


/// SECS 통신 중 사용하는 공통 메시지 모델
pub struct SecsMessage {
    pub device_id: DeviceId,
    pub system_byte: SystemByte,
    pub rbit: Rbit,
    pub payload: Secs2Message,
}

impl SecsMessage {
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

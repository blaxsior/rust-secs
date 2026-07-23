use alloc::collections::VecDeque;
use alloc::vec::Vec;

use crate::transport::error::SecsTransportError;
use crate::transport::hsms::HsmsMessage;

/// TCP byte stream을 message 단위로 조립하는 객체
///
/// - handle_read: 들어온 byte를 누적
/// - poll_message: 완성된 HSMS message를 하나 꺼냄
pub struct HsmsAssembler {
    incoming_buffer: Vec<u8>,
    outgoing_messages: VecDeque<HsmsMessage>,
}

impl HsmsAssembler {
    pub fn new() -> Self {
        Self {
            incoming_buffer: Vec::new(),
            outgoing_messages: VecDeque::new(),
        }
    }

    pub fn has_remained(&self) -> bool {
        self.incoming_buffer.len() > 0
    }

    /// 들어온 바이트를 누적하고, 완성된 메시지를 내부 큐에 쌓는다.
    pub fn handle_read(&mut self, bytes: &[u8]) -> Result<(), SecsTransportError> {
        self.incoming_buffer.extend_from_slice(bytes);
        self.process_buffer()
    }

    /// 완성된 message를 하나 꺼낸다.
    pub fn poll_message(&mut self) -> Option<HsmsMessage> {
        self.outgoing_messages.pop_front()
    }

    /// T8 timeout 등에 의해 기존에 쌓인 버퍼를 clear
    pub fn clear(&mut self) {
        self.incoming_buffer.clear();
    }

    fn process_buffer(&mut self) -> Result<(), SecsTransportError> {
        // 여러 메시지를 생성할 만큼 들어온 데이터가 큰 경우 고려
        loop {
            if self.incoming_buffer.len() < 4 {
                return Ok(());
            }

            let len = u32::from_be_bytes([
                self.incoming_buffer[0],
                self.incoming_buffer[1],
                self.incoming_buffer[2],
                self.incoming_buffer[3],
            ]) as usize;

            let total = 4 + len;
            if self.incoming_buffer.len() < total {
                return Ok(());
            }

            let message_bytes: Vec<u8> = self.incoming_buffer.drain(0..total).collect();
            let message = HsmsMessage::try_from(message_bytes.as_slice())?;
            self.outgoing_messages.push_back(message);
        }
    }
}

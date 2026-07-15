use alloc::vec::Vec;
use core::convert::TryFrom;

use secs_ii::{Secs2Message, item::Secs2Variant};

use crate::transport::hsms::{
    HsmsControlMessage, HsmsDataMessage, HsmsHeader, HsmsMessage, HsmsPType, HsmsSType,
};
use crate::transport::{SessionId, SystemByte};

/// HSMS frame
///
/// - 4 byte length field
/// - 10 byte header
/// - payload bytes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HsmsFrame {
    pub header: [u8; 10],
    pub message_text: Vec<u8>,
}

impl HsmsFrame {
    pub fn new(header: [u8; 10], message_text: Vec<u8>) -> Self {
        Self { header, message_text }
    }

    /// message length field에 들어갈 길이
    pub fn message_length(&self) -> usize {
        self.header.len() + self.message_text.len()
    }

    /// raw bytes 전체 길이
    pub fn length(&self) -> usize {
        4 + self.message_length()
    }

    pub fn into_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.length());
        let len = self.message_length() as u32;
        bytes.extend_from_slice(&len.to_be_bytes());
        bytes.extend_from_slice(&self.header);
        bytes.extend_from_slice(&self.message_text);
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 14 {
            return None;
        }

        let len = u32::from_be_bytes(bytes[0..4].try_into().ok()?) as usize;
        let end = 4 + len;
        let header: [u8; 10] = bytes.get(4..14)?.try_into().ok()?;
        let message_text = bytes.get(14..end)?.to_vec();
        Some(Self { header, message_text })
    }

    pub fn from_data_message(msg: &HsmsDataMessage) -> Self {
        let mut header = [0u8; 10];
        header[0..2].copy_from_slice(&msg.header.session_id.0.to_be_bytes());
        header[2] = msg.header.byte2;
        header[3] = msg.header.byte3;
        header[4] = msg.header.ptype.into();
        header[5] = msg.header.stype.into();
        header[6..10].copy_from_slice(&msg.header.system_byte.0.to_be_bytes());

        let mut message_text = Vec::new();
        msg.payload
            .body
            .encode(&mut message_text)
            .expect("secs2 encode should not fail in frame builder");

        Self { header, message_text }
    }

    pub fn from_control_message(msg: &HsmsControlMessage) -> Self {
        let mut header = [0u8; 10];
        header[0..2].copy_from_slice(&msg.header.session_id.0.to_be_bytes());
        header[2] = msg.header.byte2;
        header[3] = msg.header.byte3;
        header[4] = msg.header.ptype.into();
        header[5] = msg.header.stype.into();
        header[6..10].copy_from_slice(&msg.header.system_byte.0.to_be_bytes());

        Self {
            header,
            message_text: Vec::new(),
        }
    }

    pub fn try_into_message(&self) -> Option<HsmsMessage> {
        let header = self.try_into_header()?;
        if header.is_data() {
            Some(HsmsMessage::Data(HsmsDataMessage::new(
                header,
                self.try_into_secs2_message(header)?,
            )))
        } else {
            Some(HsmsMessage::Control(HsmsControlMessage::new(header)))
        }
    }

    pub fn try_into_header(&self) -> Option<HsmsHeader> {
        if self.header.len() != 10 {
            return None;
        }

        let session_id = SessionId(u16::from_be_bytes(self.header[0..2].try_into().ok()?));
        let byte2 = self.header[2];
        let byte3 = self.header[3];
        let ptype = HsmsPType::try_from(self.header[4]).ok()?;
        let stype = HsmsSType::try_from(self.header[5]).ok()?;
        let system_byte = SystemByte(u32::from_be_bytes(self.header[6..10].try_into().ok()?));

        Some(HsmsHeader::new(
            session_id,
            byte2,
            byte3,
            ptype,
            stype,
            system_byte,
        ))
    }

    pub fn try_into_secs2_message(&self, header: HsmsHeader) -> Option<Secs2Message> {
        if !header.is_data() {
            return None;
        }

        let body = Secs2Variant::try_from(self.message_text.as_slice()).ok()?;
        Some(Secs2Message::new(
            header.stream(),
            header.function(),
            header.need_reply(),
            body,
        ))
    }
}

trait EncodeSecs2Message {
    fn encode(&self, w: &mut Vec<u8>) -> Result<(), secs_ii::error::Secs2Error>;
}

impl EncodeSecs2Message for Secs2Variant {
    fn encode(&self, w: &mut Vec<u8>) -> Result<(), secs_ii::error::Secs2Error> {
        use secs_ii::convert::secs2::serialize::Encode;
        Encode::encode(self, w)
    }
}

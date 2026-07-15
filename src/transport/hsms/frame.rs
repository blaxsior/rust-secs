use alloc::vec::Vec;

/// HSMS 통신 중 받은 순정 데이터
pub struct HsmsFrame {
    pub header: [u8;10],
    pub data: Vec<u8>,
}
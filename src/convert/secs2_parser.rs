use std::io::Cursor;

use crate::items::base::{Secs2ItemCode, Secs2ItemType};

pub struct Secs2ItemParser {}

impl Secs2ItemParser {
    /// 입력된 byte 데이터를 secs2 형식으로 파싱한다.
    pub fn parse(data: Vec<u8>) -> Secs2ItemType {
        let mut cursor = Cursor::new(data);
        // 헤더 파싱
        // 아이템 길이 파싱
        // 아이템 타입에 따라 다르게 처리 수행

        Secs2ItemType::Char
    }

    /// 1-byte 데이터의 상위 6비트에서 Item type 획득
    fn get_item_code(byte: u8) -> Result<Secs2ItemCode, String> {
        let itembit = (byte & 0b11111100) >> 2;

        Secs2ItemCode::try_from(itembit).or_else(|_| Err(format!("failed to parse item code for {}", itembit)))
    }

    /// 1-byte 데이터의 하위 2비트에서 byte length 획득
    fn get_length_bytes_length(byte: u8) -> usize {
        (byte & 0b00000011) as usize
    } 
}

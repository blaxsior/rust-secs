use std::io::Write;

use crate::item::Secs2Variant;

///
/// Secs2Variant 객체를 byte 버퍼로 serialize 한다.
///
pub fn serialize(data: &Secs2Variant) -> Result<Vec<u8>, String> {
    let mut buffer = Vec::new();

    serialize_to(&mut buffer, data)?;
    Ok(buffer)
}

///
/// Secs2Variant 객체를 대응되는 buffer에 쓴다.
///
pub fn serialize_to<W>(buffer: &mut W, data: &Secs2Variant) -> Result<(), String>
where
    W: Write,
{
    serialize_impl(buffer, data)
}

///
/// Secs2Variant 객체의 byte 변환에 대한 구현체
///
fn serialize_impl<W>(buffer: &mut W, data: &Secs2Variant) -> Result<(), String>
where
    W: Write,
{
    let item_length = data
        .value()
        .expect("target value type must be implemented")
        .length();

    let format_code = data.format_code();

    // 3byte를 넘어서는 안됨
    if item_length > 0xFFFFFF {
        return Err(format!("item length to long: {}", item_length));
    }

    let mut header_byte = format_code.into() << 2 | ;
}

///
/// 입력된 길이를 byte 배열로 인코딩한 결과를 반환한다.
/// 
fn encode_length(len: usize) -> ([u8;3], usize) {
    // 최대 3byte 배열
    let mut buf = [0u8; 3];
}
use std::io::Write;

use crate::{error::Secs2Error, item::Secs2Variant};

pub trait Encode {
    fn encode<W: Write>(&self, w: &mut W) -> Result<(), Secs2Error>;
}

///
/// Secs2Variant 객체를 byte 버퍼로 serialize 한다.
///
pub fn serialize(data: &Secs2Variant) -> Result<Vec<u8>, Secs2Error> {
    let mut buffer = Vec::new();

    serialize_to(&mut buffer, data)?;
    Ok(buffer)
}

///
/// Secs2Variant 객체를 대응되는 buffer에 쓴다.
///
pub fn serialize_to<W>(buffer: &mut W, data: &Secs2Variant) -> Result<(), Secs2Error>
where
    W: Write,
{
    serialize_impl(buffer, data)
}

///
/// Secs2Variant 객체의 byte 변환에 대한 구현체
///
fn serialize_impl<W>(buffer: &mut W, data: &Secs2Variant) -> Result<(), Secs2Error>
where
    W: Write,
{
    let item_length = data.value()?.length();
    let format_code = data.format_code();

    // header 데이터 쓰기
    let length_bytes = encode_length(item_length)?;
    let header_byte: u8 = u8::from(format_code) << 2 | (length_bytes.len() as u8);

    buffer.write_all(&[header_byte])?;
    buffer.write_all(&length_bytes)?;

    // 실제 데이터 쓰기 작업

    if let Secs2Variant::List(item_list) = data {
        // secs2 list 타입인 경우 재귀 처리
        for item in item_list.items() {
            serialize_impl(buffer, item)?;
        }
    } else {
        // list 이외의 케이스 처리
        match data {
            Secs2Variant::List(_) => panic!("unreachable code"),
            Secs2Variant::Binary(v) => v.encode(buffer)?,
            Secs2Variant::Boolean(v) => v.encode(buffer)?,
            Secs2Variant::ASCII(v) => v.encode(buffer)?,
            Secs2Variant::Int8(v) => v.encode(buffer)?,
            Secs2Variant::Int1(v) => v.encode(buffer)?,
            Secs2Variant::Int2(v) => v.encode(buffer)?,
            Secs2Variant::Int4(v) => v.encode(buffer)?,
            Secs2Variant::Float8(v) => v.encode(buffer)?,
            Secs2Variant::Float4(v) => v.encode(buffer)?,
            Secs2Variant::UInt8(v) => v.encode(buffer)?,
            Secs2Variant::UInt1(v) => v.encode(buffer)?,
            Secs2Variant::UInt2(v) => v.encode(buffer)?,
            Secs2Variant::UInt4(v) => v.encode(buffer)?,
            Secs2Variant::Jis8 | Secs2Variant::Char => {
                return Err(Secs2Error::Unimplemented);
            }
        };
    };

    Ok(())
}

///
/// 입력된 길이를 byte 배열로 인코딩한 결과를 반환한다.
///
fn encode_length(len: usize) -> Result<Vec<u8>, Secs2Error> {
    // 3byte를 넘어서는 안됨
    if len > 0xFFFFFF {
        return Err(Secs2Error::InvalidLength(len));
    }

    let bytes = [(len >> 16) as u8, (len >> 8) as u8, len as u8];

    let start = bytes.iter().position(|&b| b != 0).unwrap_or(2);

    Ok(bytes[start..].to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    mod encode_length_test {
        use super::*;

        #[test]
        fn return_err_if_len_over_0x_ffffff() {
            let len = 0x1000000;

            encode_length(len).expect_err("must return err");
        }

        /// length = 0인 경우에도 배열이 반환되어야 함
        #[test]
        fn return_bytes_if_length_0() {
            let len = 0x00;

            let result = encode_length(len).expect("must return item");

            // length = 0이어도 byte 길이 1은 보장되어야 함
            assert_eq!(result.len(), 1);
            assert_eq!(result[0], 0);
        }

        #[test]
        fn return_bytes_normal_case() {
            let len = 0xABCDEF;

            let result = encode_length(len).expect("must return item");

            assert_eq!(result.len(), 3);
            assert_eq!(result, [0xAB, 0xCD, 0xEF]);
        }
    }

    mod serialize_test {
        use crate::item::{Secs2Item, ascii::Secs2ASCII, binary::Secs2Binary, list::Secs2List};

        use super::*;

        #[test]
        fn should_encode_data_properly_if_byte_normal() {
            let expected: Vec<u8> = vec![
                0x01, 0x02, 0x21, 0x02, 0x0B, 0x0C, 0x41, 0x05, 0x68, 0x65, 0x6C, 0x6C, 0x6F,
            ];

            let variant = Secs2List::new(vec![
                Secs2Binary::new(vec![11u8, 12u8]).as_enum(),
                Secs2ASCII::new(String::from("hello")).as_enum(),
            ])
            .as_enum();

            let mut buf = Vec::new();
            serialize_to(&mut buf, &variant).expect("must be encoded");

            assert_eq!(buf, expected);
        }
    }
}

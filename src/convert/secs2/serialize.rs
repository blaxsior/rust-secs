use crate::{error::Secs2Error, item::Secs2Variant};
use alloc::vec::Vec;

pub trait Encode {
    fn encode(&self, w: &mut Vec<u8>) -> Result<(), Secs2Error>;
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
pub fn serialize_to(buffer: &mut Vec<u8>, data: &Secs2Variant) -> Result<(), Secs2Error> {
    serialize_impl(buffer, data)
}

///
/// Secs2Variant 객체의 byte 변환에 대한 구현체
///
fn serialize_impl(buffer: &mut Vec<u8>, data: &Secs2Variant) -> Result<(), Secs2Error> {
    let item_length = data.value()?.length();
    let format_code = data.format_code();

    // header 데이터 쓰기
    let (length_bytes, length_len) = encode_length(item_length)?;
    let valid_length_slice = &length_bytes[..length_len];

    let header_byte: u8 = (u8::from(format_code) << 2) | (length_len as u8);

    buffer.push(header_byte);
    buffer.extend_from_slice(valid_length_slice);

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
fn encode_length(len: usize) -> Result<([u8; 3], usize), Secs2Error> {
    // 3byte를 넘어서는 안됨 (0xFFFFFF = 16,777,215 바이트)
    if len > 0xFFFFFF {
        return Err(Secs2Error::InvalidLength(len));
    }

    // 1. 빅엔디안 형태로 3바이트에 쪼개 넣기
    let bytes = [(len >> 16) as u8, (len >> 8) as u8, len as u8];

    // 2. 0이 아닌 최초의 위치 찾기 (길이가 0일 때는 최소 1바이트는 써야 하므로 unwrap_or(2) 유지)
    let start = bytes.iter().position(|&b| b != 0).unwrap_or(2);

    // 3. 유효한 바이트들만 앞으로 정렬하여 새 배열 생성
    let mut result = [0u8; 3];
    let valid_len = 3 - start;
    result[..valid_len].copy_from_slice(&bytes[start..]);

    // (배열 데이터, 실제 유효한 길이) 반환
    Ok((result, valid_len))
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

            let (bytes, valid_len) = encode_length(len).expect("must return item");

            // 💡 튜플에서 받아온 실제 유효 길이만큼 슬라이스를 떼어내서 검증합니다.
            let result_slice = &bytes[..valid_len];

            // length = 0이어도 byte 길이 1은 보장되어야 함
            assert_eq!(result_slice.len(), 1);
            assert_eq!(result_slice[0], 0);
        }

        #[test]
        fn return_bytes_normal_case() {
            let len = 0xABCDEF;

            let (bytes, valid_len) = encode_length(len).expect("must return item");
            let result_slice = &bytes[..valid_len];

            assert_eq!(result_slice.len(), 3);
            assert_eq!(result_slice, [0xAB, 0xCD, 0xEF]);
        }
    }

    mod serialize_test {
        use super::*;

        #[test]
        fn should_encode_data_properly_if_byte_normal() {
            let expected: Vec<u8> = vec![
                0x01, 0x02, 0x21, 0x02, 0x0B, 0x0C, 0x41, 0x05, 0x68, 0x65, 0x6C, 0x6C, 0x6F,
            ];

            let variant = Secs2Variant::list(vec![
                Secs2Variant::binary(vec![11u8, 12u8]),
                Secs2Variant::ascii(String::from("hello")),
            ]);

            let mut buf = Vec::new();
            serialize_to(&mut buf, &variant).expect("must be encoded");

            assert_eq!(buf, expected);
        }
    }
}

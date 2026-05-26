use crate::{
    item::{
        Secs2FormatCode, Secs2Item, Secs2Variant, ascii::Secs2ASCII, binary::Secs2Binary,
        boolean::Secs2Boolean, float4::Secs2Float4, float8::Secs2Float8, int1::Secs2Int1,
        int2::Secs2Int2, int4::Secs2Int4, int8::Secs2Int8, list::Secs2List, uint1::Secs2Uint1,
        uint2::Secs2Uint2, uint4::Secs2Uint4, uint8::Secs2Uint8,
    }
};

use alloc::string::String;
use alloc::vec::Vec;

pub fn parse<T>(data: &T) -> Result<Secs2Variant, String>
where
    T: AsRef<[u8]>,
{
    let mut stream = data.as_ref();
    parse_impl(&mut stream)
}

/// 입력된 데이터를 secs2 형식으로 파싱한다.
fn parse_impl(reader: &mut &[u8]) -> Result<Secs2Variant, String> {
    // 1. header 첫번째 라인 획득 (read_u8 대체)
    if reader.is_empty() {
        return Err("Unexpected EOF when reading header".into());
    }
    let (head, tail) = reader.split_at(1);
    *reader = tail;
    let header_byte = head[0];

    // header 분석
    let format_code = get_format_code(header_byte)?;
    let length_bytes_no = get_length_bytes_number(header_byte)?;

    // 내부 헬퍼 함수도 reader: &mut &[u8]을 받도록 수정되어야 합니다.
    let item_length_bytes = read_item_length_bytes(reader, length_bytes_no)?;
    let item_length = get_item_length(&item_length_bytes);

    // list 인 경우 자식으로 처리
    if let Secs2FormatCode::List = format_code {
        let mut items: Vec<Secs2Variant> = Vec::with_capacity(item_length);
        for _ in 1..=item_length {
            items.push(parse_impl(reader)?); // 가변 reader 전파
        }

        Ok(Secs2List::new(items).as_enum())
    } else {
        // 2. 아닌 경우 byte 데이터를 읽어 대상 아이템 생성 (read_exact 대체)
        if reader.len() < item_length {
            return Err(format!(
                "Unexpected EOF: expected {} bytes, but got {}",
                item_length,
                reader.len()
            ));
        }
        let (buf, tail) = reader.split_at(item_length);
        *reader = tail; // 읽은 만큼 포인터 전진

        let item: Secs2Variant = match format_code {
            Secs2FormatCode::List => panic!("unreachable code"),
            // 이제 buf 자체가 &[u8] 슬라이스이므로 .as_slice() 없이 바로 주입 가능합니다.
            Secs2FormatCode::ASCII => Secs2ASCII::try_from(buf)?.as_enum(),
            Secs2FormatCode::Binary => Secs2Binary::try_from(buf)?.as_enum(),
            Secs2FormatCode::Boolean => Secs2Boolean::try_from(buf)?.as_enum(),
            Secs2FormatCode::UInt8 => Secs2Uint8::try_from(buf)?.as_enum(),
            Secs2FormatCode::UInt1 => Secs2Uint1::try_from(buf)?.as_enum(),
            Secs2FormatCode::UInt2 => Secs2Uint2::try_from(buf)?.as_enum(),
            Secs2FormatCode::UInt4 => Secs2Uint4::try_from(buf)?.as_enum(),
            Secs2FormatCode::Float8 => Secs2Float8::try_from(buf)?.as_enum(),
            Secs2FormatCode::Float4 => Secs2Float4::try_from(buf)?.as_enum(),
            Secs2FormatCode::Int8 => Secs2Int8::try_from(buf)?.as_enum(),
            Secs2FormatCode::Int1 => Secs2Int1::try_from(buf)?.as_enum(),
            Secs2FormatCode::Int2 => Secs2Int2::try_from(buf)?.as_enum(),
            Secs2FormatCode::Int4 => Secs2Int4::try_from(buf)?.as_enum(),
            Secs2FormatCode::Jis8 | Secs2FormatCode::Char => {
                return Err("not implemented type".into());
            }
        };
        Ok(item)
    }
}
/// 1-byte 데이터의 상위 6비트에서 Item type 획득
fn get_format_code(byte: u8) -> Result<Secs2FormatCode, String> {
    let itembit = (byte & 0b11111100) >> 2;

    Secs2FormatCode::try_from(itembit)
        .or_else(|_| Err(format!("no matched format for {:#o}", itembit)))
}

/// 1-byte 데이터의 하위 2비트에서 byte length 획득
fn get_length_bytes_number(byte: u8) -> Result<usize, String> {
    let bytes_length_no = (byte & 0b00000011) as usize;
    if bytes_length_no < 1 || bytes_length_no > 3 {
        Err(format!(
            "length bytes number must be in [1..3], found {}",
            bytes_length_no
        ))
    } else {
        Ok(bytes_length_no)
    }
}

///
/// item length bytes을 읽는다.
///
fn read_item_length_bytes(
    reader: &mut &[u8], // R: Read 대신 가변 슬라이스로 변경
    length_bytes_no: usize,
) -> Result<Vec<u8>, String> {
    if length_bytes_no < 1 || length_bytes_no > 3 {
        return Err(format!(
            "length bytes number must be in [1..3], found {}",
            length_bytes_no
        ));
    }

    // 슬라이스에 남은 바이트가 충분한지 체크
    if reader.len() < length_bytes_no {
        return Err(format!(
            "Unexpected EOF: need {} bytes for length field, but only {} bytes left",
            length_bytes_no,
            reader.len()
        ));
    }

    // 지정된 길이만큼 정확히 잘라냅니다.
    let (buf, tail) = reader.split_at(length_bytes_no);
    *reader = tail; // 읽은 만큼 포인터 이동

    // 기존 인터페이스 호환을 위해 잘라낸 슬라이스를 Vec으로 변환하여 반환
    Ok(buf.to_vec())
}

/// item의 길이를 얻는다. list는 item 개수, list 이외는 byte 길이에 해당한다.
fn get_item_length(bytes: &[u8]) -> usize {
    bytes.iter().fold(0, |acc, &b| (acc << 8) | (b as usize))
}

#[cfg(test)]
mod tests {
    use super::*;

    mod get_format_code_test {
        use super::*;

        #[test]
        fn list_code() {
            let b: u8 = 0b00 << 2;
            let expected = Secs2FormatCode::List;
            assert_eq!(get_format_code(b).unwrap(), expected);
        }

        #[test]
        fn binary_code() {
            let b: u8 = 0o10 << 2;
            let expected = Secs2FormatCode::Binary;
            assert_eq!(get_format_code(b).unwrap(), expected);
        }

        #[test]
        fn boolean_code() {
            let b: u8 = 0o11 << 2;
            let expected = Secs2FormatCode::Boolean;
            assert_eq!(get_format_code(b).unwrap(), expected);
        }

        #[test]
        fn ascii_code() {
            let b: u8 = 0o20 << 2;
            let expected = Secs2FormatCode::ASCII;
            assert_eq!(get_format_code(b).unwrap(), expected);
        }

        #[test]
        fn jis8_code() {
            let b: u8 = 0o21 << 2;
            let expected = Secs2FormatCode::Jis8;
            assert_eq!(get_format_code(b).unwrap(), expected);
        }

        #[test]
        fn char_code() {
            let b: u8 = 0o22 << 2;
            let expected = Secs2FormatCode::Char;
            assert_eq!(get_format_code(b).unwrap(), expected);
        }

        #[test]
        fn int8_code() {
            let b: u8 = 0o30 << 2;
            let expected = Secs2FormatCode::Int8;
            assert_eq!(get_format_code(b).unwrap(), expected);
        }

        #[test]
        fn int1_code() {
            let b: u8 = 0o31 << 2;
            let expected = Secs2FormatCode::Int1;
            assert_eq!(get_format_code(b).unwrap(), expected);
        }

        #[test]
        fn int2_code() {
            let b: u8 = 0o32 << 2;
            let expected = Secs2FormatCode::Int2;
            assert_eq!(get_format_code(b).unwrap(), expected);
        }

        #[test]
        fn int4_code() {
            let b: u8 = 0o34 << 2;
            let expected = Secs2FormatCode::Int4;
            assert_eq!(get_format_code(b).unwrap(), expected);
        }

        #[test]
        fn float8_code() {
            let b: u8 = 0o40 << 2;
            let expected = Secs2FormatCode::Float8;
            assert_eq!(get_format_code(b).unwrap(), expected);
        }

        #[test]
        fn float4_code() {
            let b: u8 = 0o44 << 2;
            let expected = Secs2FormatCode::Float4;
            assert_eq!(get_format_code(b).unwrap(), expected);
        }

        #[test]
        fn uint8_code() {
            let b: u8 = 0o50 << 2;
            let expected = Secs2FormatCode::UInt8;
            assert_eq!(get_format_code(b).unwrap(), expected);
        }

        #[test]
        fn uint1_code() {
            let b: u8 = 0o51 << 2;
            let expected = Secs2FormatCode::UInt1;
            assert_eq!(get_format_code(b).unwrap(), expected);
        }

        #[test]
        fn uint2_code() {
            let b: u8 = 0o52 << 2;
            let expected = Secs2FormatCode::UInt2;
            assert_eq!(get_format_code(b).unwrap(), expected);
        }

        #[test]
        fn uint4_code() {
            let b: u8 = 0o54 << 2;
            let expected = Secs2FormatCode::UInt4;
            assert_eq!(get_format_code(b).unwrap(), expected);
        }
    }

    #[cfg(test)]
    mod read_item_length_bytes_test {
        use super::*;

        // 💡 이제 std::io::Cursor 임포트가 필요 없습니다.

        #[test]
        fn should_return_err_if_length_number_smaller_than_1() {
            let err_num = 0;
            let buf = [0u8; 2];
            let mut stream = &buf[..]; // Cursor::new(&buf) 대신 슬라이스로 만듭니다.

            let result = read_item_length_bytes(&mut stream, err_num);
            let msg = result.expect_err("must err if num out of range [1,3]");
            assert!(msg.contains("length bytes number must be in"));
        }

        #[test]
        fn should_return_err_if_length_number_bigger_than_3() {
            let err_num = 4;
            let buf = [0u8; 2];
            let mut stream = &buf[..];

            let result = read_item_length_bytes(&mut stream, err_num);
            let msg = result.expect_err("must err if num out of range [1,3]");
            assert!(msg.contains("length bytes number must be in"));
        }

        // buffer is not enough for len_number
        #[test]
        fn should_return_err_if_buffer_length_not_enough() {
            let len_number = 3;
            let buf = [0u8; 2];
            let mut stream = &buf[..];

            let result = read_item_length_bytes(&mut stream, len_number);
            let msg = result.expect_err("buffer size is not enough. but enough");

            // 💡 수정된 함수의 에러 메시지 형식에 맞춰 "Unexpected EOF"로 검증합니다.
            assert!(msg.contains("Unexpected EOF"));
        }

        #[test]
        fn should_return_ok_if_buffer_length_enough() {
            let len_number = 3;
            let buf = [0xABu8, 0xCDu8, 0xEFu8];
            let mut stream = &buf[..];

            let result = read_item_length_bytes(&mut stream, len_number);
            let length_bytes = result.expect("buffer size is not enough");
            assert_eq!(len_number, length_bytes.len());

            for i in 0..len_number {
                assert_eq!(buf[i], length_bytes[i]);
            }

            // 💡 덤으로 stream이 완전히 소비되어 빈 슬라이스가 되었는지 검증도 가능해집니다.
            assert!(stream.is_empty());
        }
    }

    #[cfg(test)]
    mod get_item_length_test {
        use super::*;

        #[test]
        fn should_return_buffer_as_big_endian_usize() {
            let bytes: [u8; 3] = [0xAB, 0xCD, 0xEF];
            let expected: usize = 0xABCDEF;

            let result = get_item_length(&bytes);

            assert_eq!(result, expected);
        }
    }
}

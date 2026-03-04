use std::io::{Cursor, Read};

use crate::{
    item::{
        Secs2FormatCode, Secs2Item, Secs2Variant, ascii::Secs2ASCII, binary::Secs2Binary,
        boolean::Secs2Boolean, float4::Secs2Float4, float8::Secs2Float8, int1::Secs2Int1,
        int2::Secs2Int2, int4::Secs2Int4, int8::Secs2Int8, list::Secs2List, uint1::Secs2Uint1,
        uint2::Secs2Uint2, uint4::Secs2Uint4, uint8::Secs2Uint8,
    },
    util::cursor_extensions::CursorReadExt,
};

pub fn parse<T>(data: &T) -> Result<Secs2Variant, String>
where
    T: AsRef<[u8]>,
{
    let mut cursor = Cursor::new(data.as_ref());
    parse_impl(&mut cursor)
}

/// 입력된 데이터를 secs2 형식으로 파싱한다.
fn parse_impl<T: AsRef<[u8]>>(cursor: &mut Cursor<T>) -> Result<Secs2Variant, String> {
    // header 첫번째 라인 획득
    let header_byte = cursor.read_u8()?;

    // header 분석
    let format_code = get_format_code(header_byte)?;
    let length_bytes_no = get_length_bytes_number(header_byte)?;
    let item_length_bytes = read_item_length_bytes(cursor, length_bytes_no)?;
    let item_length = get_item_length(&item_length_bytes);

    // list 인 경우 자식으로 처리
    if let Secs2FormatCode::List = format_code {
        let mut items: Vec<Secs2Variant> = Vec::with_capacity(item_length);
        for _ in 1..=item_length {
            items.push(parse_impl(cursor)?);
        }

        Ok(Secs2List::new(items).as_enum())
    } else {
        // 아닌 경우 byte 데이터를 읽어 대상 아이템 생성
        let mut buf = vec![0u8; item_length];
        cursor
            .read_exact(&mut buf)
            .map_err(|e| format!("error occurred when reading bytes: {:#?}", e))?;

        let item: Secs2Variant = match format_code {
            Secs2FormatCode::ASCII => Secs2ASCII::try_from(buf.as_slice())?.as_enum(),
            Secs2FormatCode::Binary => Secs2Binary::try_from(buf.as_slice())?.as_enum(),
            Secs2FormatCode::Boolean => Secs2Boolean::try_from(buf.as_slice())?.as_enum(),
            Secs2FormatCode::UInt8 => Secs2Uint8::try_from(buf.as_slice())?.as_enum(),
            Secs2FormatCode::UInt1 => Secs2Uint1::try_from(buf.as_slice())?.as_enum(),
            Secs2FormatCode::UInt2 => Secs2Uint2::try_from(buf.as_slice())?.as_enum(),
            Secs2FormatCode::UInt4 => Secs2Uint4::try_from(buf.as_slice())?.as_enum(),
            Secs2FormatCode::Float8 => Secs2Float8::try_from(buf.as_slice())?.as_enum(),
            Secs2FormatCode::Float4 => Secs2Float4::try_from(buf.as_slice())?.as_enum(),
            Secs2FormatCode::Int8 => Secs2Int8::try_from(buf.as_slice())?.as_enum(),
            Secs2FormatCode::Int1 => Secs2Int1::try_from(buf.as_slice())?.as_enum(),
            Secs2FormatCode::Int2 => Secs2Int2::try_from(buf.as_slice())?.as_enum(),
            Secs2FormatCode::Int4 => Secs2Int4::try_from(buf.as_slice())?.as_enum(),
            _ => {
                // must not reach
                return Err("not implemented type".into());
            }
        };
        return Ok(item);
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
fn read_item_length_bytes<T: AsRef<[u8]>>(
    cursor: &mut Cursor<T>,
    length_bytes_no: usize,
) -> Result<Vec<u8>, String> {
    if length_bytes_no < 1 || length_bytes_no > 3 {
        return Err(format!(
            "length bytes number must be in [1..3], found {}",
            length_bytes_no
        ));
    }
    
    let mut buf = vec![0u8; length_bytes_no];
    cursor
        .read_exact(&mut buf)
        .map_err(|e| format!("error occurred when reading bytes: {:#?}", e))?;
    Ok(buf)
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

    mod get_length_bytes_number_test {
        use crate::convert::secs2_converter::get_length_bytes_number;

        #[test]
        fn return_ok_if_byte_between_1_and_3() {
            let b = 0b10010101;
            let result = get_length_bytes_number(b).unwrap();
            assert_eq!(result, 1);

            let b = 0b10100110;
            let result = get_length_bytes_number(b).unwrap();
            assert_eq!(result, 2);

            let b = 0b11100111;
            let result = get_length_bytes_number(b).unwrap();
            assert_eq!(result, 3);
        }

        /// 명세에 의해 length bytes number은 반드시 1 ~ 3 사이의 값을 가져야 함.
        /// 0은 허용되지 않음
        #[test]
        fn return_err_if_byte_be_0() {
            let b = 0b11100100;
            let result = get_length_bytes_number(b);
            assert!(
                result.is_err(),
                "length bytes number should between 1 ~ 3 by SECS-II specification"
            );
        }
    }

    mod read_item_length_bytes_test {
        use super::*;
        use std::io::Cursor;

        #[test]
        fn should_return_err_if_length_number_smaller_than_1() {
            let err_num = 0;
            let buf = [0u8; 2];
            let mut cursor = Cursor::new(&buf);

            let result = read_item_length_bytes(&mut cursor, err_num);
            let msg = result.expect_err("must err if num out of range [1,3]");
            assert!(msg.contains("length bytes number must be in"));
        }

        #[test]
        fn should_return_err_if_length_number_bigger_than_3() {
            let err_num = 4;
            let buf = [0u8; 2];
            let mut cursor = Cursor::new(&buf);

            let result = read_item_length_bytes(&mut cursor, err_num);
            let msg = result.expect_err("must err if num out of range [1,3]");
            assert!(msg.contains("length bytes number must be in"));
        }

        // buffer is not enough for len_number
        #[test]
        fn should_return_err_if_buffer_length_not_enough() {
            let len_number = 3;
            let buf = [0u8; 2];
            let mut cursor = Cursor::new(&buf);

            let result = read_item_length_bytes(&mut cursor, len_number);
            let msg = result.expect_err("expect: buffer size is not enough. but enough");
            assert!(msg.contains("error occurred when reading bytes"));
        }
    }

}

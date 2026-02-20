use std::io::{Cursor, Read};

use crate::{
    item::{
        Secs2FormatCode, Secs2Item, Secs2Variant, ascii::Secs2ASCII, binary::Secs2Binary, boolean::Secs2Boolean, float4::Secs2Float4, float8::Secs2Float8, int1::Secs2Int1, int2::Secs2Int2, int4::Secs2Int4, int8::Secs2Int8, list::Secs2List, uint1::Secs2Uint1, uint2::Secs2Uint2, uint4::Secs2Uint4, uint8::Secs2Uint8
    },
    util::cursor_extensions::{CursorReadExt},
};

pub fn parse<T>(data: T) -> Result<Secs2Variant, String>
where
    T: AsRef<[u8]>,
{
    let mut cursor = Cursor::new(data);
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
            let child = parse_impl(cursor)?;
            items.push(child);
        }

        return Ok(Secs2List::new(items).as_enum());
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
                panic!("not implemented type");
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

fn read_and_convert<CT: Read, T, F>(
    cursor: &mut CT,
    item_length: usize,
    conv: F,
) -> Result<T, String>
where
    F: FnOnce(&[u8]) -> Result<T, &'static str>,
{
    let mut buf = vec![0u8; item_length];
    cursor.read_exact(&mut buf).map_err(|e| e.to_string())?;
    conv(&buf).map_err(|e| e.to_string())
}

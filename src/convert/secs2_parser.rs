use std::fmt::format;
use std::io::{Cursor, Read};

use crate::items::ascii::Secs2ASCIIBody;
use crate::items::base::{Secs2FormatCode, Secs2Item, Secs2ItemBody};
use crate::items::list::Secs2ListBody;
use crate::util::cursor_extension::CursorReadExt;

///byte 배열을 대응되는 Secs2Item으로 파싱하는 구조체
pub struct Secs2ItemParser {}

impl Secs2ItemParser {
    // byte 배열 형식으로 입력된 데이터를 처리
    pub fn parse(data: impl AsRef<[u8]>) -> Result<Secs2Item, String> {
        let mut cursor = Cursor::new(data);
        Self::parseImpl(&mut cursor)
    }

    /// 입력된 데이터를 secs2 형식으로 파싱한다.
    fn parseImpl<T: Read>(cursor: &mut T)-> Result<Secs2Item, String> {
        // header 첫번째 라인 획득
        let header_byte = cursor.read_u8()?;

        // header 분석
        let format_code = Self::get_format_code(header_byte)?;
        let bytes_length_no = Self::get_length_bytes_number(header_byte)?;
        let item_length = Self::get_item_length(cursor, bytes_length_no)?;

        match format_code {
            Secs2FormatCode::List => {
                let mut items = Vec::with_capacity(item_length);
                for _ in 0..item_length {
                    items.push(Self::parseImpl(cursor)?);
                }
                Ok(Secs2Item::List(Secs2ListBody::new(items)))
            }
            Secs2FormatCode::ASCII => read_and_convert(cursor, item_length, |buf| Ok( Secs2ASCIIBody::try_from(buf)?.as_enum())),
            Secs2FormatCode::Binary => {
                let mut buf = vec![0u8; item_length];
                cursor.read_exact(&mut buf).or_else(|e| Err(e.to_string()))?;
                let body = Secs2ASCIIBody::try_from(buf.as_slice())?;
                Ok(body.as_enum())
            }
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

    /// item의 길이를 얻는다. list는 item 개수, list 이외는 byte 길이에 해당한다.
    fn get_item_length<T: AsRef<[u8]>>(
        cursor: &mut Cursor<T>,
        bytes_length_no: usize,
    ) -> Result<usize, String> {
        if bytes_length_no < 1 || bytes_length_no > 3 {
            return Err(format!(
                "length bytes number must be in [1..3], found {}",
                bytes_length_no
            ));
        }

        let mut buf = [0u8; 3]; // 최대 3바이트만 사용

        // bytes_length_no 만큼만 읽기
        cursor
            .read_exact(&mut buf[..bytes_length_no])
            .map_err(|e| format!("error occurred when parsing item length: {:#?}", e))?;

        let mut result = 0usize;

        for &b in &buf[..bytes_length_no] {
            result = (result << 8) | (b as usize);
        }

        Ok(result)
    }
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
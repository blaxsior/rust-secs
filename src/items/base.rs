use num_enum::TryFromPrimitive;

use crate::items::{
    ascii::Secs2ASCIIBody, binary::Secs2BinaryBody, boolean::Secs2BooleanBody, float4::Secs2Float4Body,
    float8::Secs2Float8Body, int1::Secs2Int1Body, int2::Secs2Int2Body, int4::Secs2Int4Body, int8::Secs2Int8Body,
    list::Secs2ListBody, uint1::Secs2Uint1Body, uint2::Secs2Uint2Body, uint4::Secs2Uint4Body, uint8::Secs2Uint8Body,
};

/// Secs-II 규격에 맞는 아이템 요소를 의미하는 trait
pub trait Secs2ItemBody: ToString {
    fn as_enum(self) -> Secs2Item;
    // fn to_string_impl(&self) -> String;

    /// item의 길이를 반환한다.
    fn item_length(&self) -> usize;

    // length_bytes_number(최대 3byte)를 반환한다.
    fn length_bytes(&self) -> Result<Vec<u8>, String> {
        let mut item_len = self.item_length();

        if item_len > 0xFFFFFF {
            return Err(format!(
                "data length {} is too long. length must be under 0xFFFFFF",
                item_len
            ));
        }

        let mut bits = Vec::new();
        while item_len > 0 {
            bits.push((item_len & 0xFF) as u8);
            item_len >>= 8;
        }
        bits.reverse();

        Ok(bits)
    }
}

/// Secs2Item 객체를 표현하는 Enum 객체
pub enum Secs2Item {
    List(Secs2ListBody),
    Binary(Secs2BinaryBody),
    Boolean(Secs2BooleanBody),
    ASCII(Secs2ASCIIBody),
    /// JIS-8 타입. 현재 미구현
    Jis8,
    /// 2-byte char. 현재 미구현
    Char,
    Int8(Secs2Int8Body),
    Int1(Secs2Int1Body),
    Int2(Secs2Int2Body),
    Int4(Secs2Int4Body),
    Float8(Secs2Float8Body),
    Float4(Secs2Float4Body),
    UInt8(Secs2Uint8Body),
    UInt1(Secs2Uint1Body),
    UInt2(Secs2Uint2Body),
    UInt4(Secs2Uint4Body),
}

impl Secs2Item {
    fn value(&mut self) -> Result<&mut dyn Secs2ItemBody, &'static str> {
        match self {
            Self::List(v) => Ok(v),
            Self::Binary(v) => Ok(v),
            Self::Boolean(v) => Ok(v),
            Self::ASCII(v) => Ok(v),
            Self::Int8(v) => Ok(v),
            Self::Int1(v) => Ok(v),
            Self::Int2(v) => Ok(v),
            Self::Int4(v) => Ok(v),
            Self::Float8(v) => Ok(v),
            Self::Float4(v) => Ok(v),
            Self::UInt8(v) => Ok(v),
            Self::UInt1(v) => Ok(v),
            Self::UInt2(v) => Ok(v),
            Self::UInt4(v) => Ok(v),
            _ => Err("target type is not implemented yet"),
        }
    }

    /// 현재 Item에 대한 format code를 반환
    fn format_code(&mut self) -> Secs2FormatCode {
        match self {
            Self::List(_) => Secs2FormatCode::List,
            Self::Binary(_) => Secs2FormatCode::Binary,
            Self::Boolean(_) => Secs2FormatCode::Boolean,
            Self::ASCII(_) => Secs2FormatCode::ASCII,
            Self::Jis8 => Secs2FormatCode::Jis8,
            Self::Char => Secs2FormatCode::Char,
            Self::Int8(_) => Secs2FormatCode::Int8,
            Self::Int1(_) => Secs2FormatCode::Int1,
            Self::Int2(_) => Secs2FormatCode::Int2,
            Self::Int4(_) => Secs2FormatCode::Int4,
            Self::Float8(_) => Secs2FormatCode::Float8,
            Self::Float4(_) => Secs2FormatCode::Float4,
            Self::UInt8(_) => Secs2FormatCode::UInt8,
            Self::UInt1(_) => Secs2FormatCode::UInt1,
            Self::UInt2(_) => Secs2FormatCode::UInt2,
            Self::UInt4(_) => Secs2FormatCode::UInt4,
        }
    }
}

/// SECS2 아이템 타입 코드를 표현하는 enum
#[derive(Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum Secs2FormatCode {
    List = 0o00,
    Binary = 0o10,
    Boolean = 0o11,
    ASCII = 0o20,
    Jis8 = 0o21,
    Char = 0o22,
    Int8 = 0o30,
    Int1 = 0o31,
    Int2 = 0o32,
    Int4 = 0o34,
    Float8 = 0o40,
    Float4 = 0o44,
    UInt8 = 0o50,
    UInt1 = 0o51,
    UInt2 = 0o52,
    UInt4 = 0o54,
}



// impl TryFrom<Vec<u8>> for Secs2Item {
//     type Error = &'static str;

//     fn try_from(buf: Vec<u8>) -> Result<Self, Self::Error> {
//     }
// }

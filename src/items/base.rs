use crate::items::{
    ascii::Secs2ASCII, binary::Secs2Binary, boolean::Secs2Boolean, float4::Secs2Float4,
    float8::Secs2Float8, int1::Secs2Int1, int2::Secs2Int2, int4::Secs2Int4, int8::Secs2Int8,
    list::Secs2List, uint1::Secs2Uint1, uint2::Secs2Uint2, uint4::Secs2Uint4, uint8::Secs2Uint8,
};

/// Secs-II 규격에 맞는 아이템 요소를 의미하는 trait
pub trait Secs2Item: ToString {
    fn as_enum(self) -> Secs2ItemType;
    // fn to_string_impl(&self) -> String;

    /// 최대 3byte로 표현되는 item의 길이를 반환한다.
    fn item_length(&self) -> usize;

    // item length byte(최대 3byte)를 반환한다.
    fn item_length_bytes(&self) -> Result<Vec<u8>, String> {
        let mut item_len = self.item_length();

        if item_len > 0xFFFFFF {
            return Err(format!("data length {} is too long. length must be under 0xFFFFFF", item_len));
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

/// Secs2Item 객체를 표현하는 Enum
///
#[repr(u8)]
pub enum Secs2ItemType {
    List(Secs2List) = 0o00,
    Binary(Secs2Binary) = 0o10,
    Boolean(Secs2Boolean) = 0o11,
    ASCII(Secs2ASCII) = 0o20,
    /// JIS-8 타입. 현재 미구현
    Jis8 = 0o21,
    /// 2-byte char. 현재 미구현
    Char = 0o22,
    Int8(Secs2Int8) = 0o30,
    Int1(Secs2Int1) = 0o31,
    Int2(Secs2Int2) = 0o32,
    Int4(Secs2Int4) = 0o34,
    Float8(Secs2Float8) = 0o40,
    Float4(Secs2Float4) = 0o44,
    UInt8(Secs2Uint8) = 0o50,
    UInt1(Secs2Uint1) = 0o51,
    UInt2(Secs2Uint2) = 0o52,
    UInt4(Secs2Uint4) = 0o54,
}

impl Secs2ItemType {
    fn value(&mut self) -> Result<&mut dyn Secs2Item, &'static str> {
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
}

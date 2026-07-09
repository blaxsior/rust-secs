pub mod ascii;
pub mod binary;
pub mod boolean;
pub mod char;
pub mod float4;
pub mod float8;
pub mod int1;
pub mod int2;
pub mod int4;
pub mod int8;
pub mod jis8;
pub mod list;
pub mod uint1;
pub mod uint2;
pub mod uint4;
pub mod uint8;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{
    convert::secs2::serialize::Encode,
    error::Secs2Error,
    item::{
        ascii::{Secs2ASCII, Secs2ASCIIItem},
        binary::{Secs2Binary, Secs2BinaryItem},
        boolean::{Secs2Boolean, Secs2BooleanItem2},
        float4::{Secs2Float4, Secs2Float4Item},
        float8::{Secs2Float8, Secs2Float8Item},
        int1::{Secs2Int1, Secs2Int1Item},
        int2::{Secs2Int2, Secs2Int2Item},
        int4::{Secs2Int4, Secs2Int4Item},
        int8::{Secs2Int8, Secs2Int8Item},
        list::{Secs2List, Secs2ListItem},
        uint1::{Secs2Uint1, Secs2Uint1Item},
        uint2::{Secs2Uint2, Secs2Uint2Item},
        uint4::{Secs2Uint4, Secs2Uint4Item},
        uint8::{Secs2Uint8, Secs2Uint8Item},
    },
};

///
/// Secs-II 타입 객체를 표현하는 enum 클래스
///
#[derive(Debug)]
pub enum Secs2Variant {
    List(Secs2List),
    Binary(Secs2Binary),
    Boolean(Secs2Boolean),
    ASCII(Secs2ASCII),
    /// JIS-8 타입. 현재 미구현
    Jis8,
    /// 2-byte char. 현재 미구현
    Char,
    Int8(Secs2Int8),
    Int1(Secs2Int1),
    Int2(Secs2Int2),
    Int4(Secs2Int4),
    Float8(Secs2Float8),
    Float4(Secs2Float4),
    UInt8(Secs2Uint8),
    UInt1(Secs2Uint1),
    UInt2(Secs2Uint2),
    UInt4(Secs2Uint4),
}

impl Secs2Variant {
    pub fn value(&self) -> Result<&dyn Secs2Item, Secs2Error> {
        match self {
            Self::List(v) => Ok(v),
            Self::Binary(v) => Ok(v),
            Self::Boolean(v) => Ok(v),
            Self::ASCII(v) => Ok(v),
            Self::Jis8 => Err(Secs2Error::Unimplemented),
            Self::Char => Err(Secs2Error::Unimplemented),
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
        }
    }

    ///
    /// 현재 타입에 맞는 format code를 반환한다.
    ///
    pub fn format_code(&self) -> Secs2FormatCode {
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

    // 이하 생성자 정의
    pub fn ascii<I: Into<Secs2ASCIIItem>>(item: I) -> Secs2Variant {
        Secs2ASCII::new(item.into()).as_enum()
    }

    pub fn binary<I: Into<Secs2BinaryItem>>(item: I) -> Secs2Variant {
        Secs2Binary::new(item.into()).as_enum()
    }

    pub fn boolean<I: Into<Secs2BooleanItem2>>(item: I) -> Secs2Variant {
        Secs2Boolean::from(item.into()).as_enum()
    }

    pub fn float4<I: Into<Secs2Float4Item>>(item: I) -> Secs2Variant {
        Secs2Float4::new(item.into()).as_enum()
    }

    pub fn float8<I: Into<Secs2Float8Item>>(item: I) -> Secs2Variant {
        Secs2Float8::new(item.into()).as_enum()
    }

    pub fn int1<I: Into<Secs2Int1Item>>(item: I) -> Secs2Variant {
        Secs2Int1::new(item.into()).as_enum()
    }

    pub fn int2<I: Into<Secs2Int2Item>>(item: I) -> Secs2Variant {
        Secs2Int2::new(item.into()).as_enum()
    }

    pub fn int4<I: Into<Secs2Int4Item>>(item: I) -> Secs2Variant {
        Secs2Int4::new(item.into()).as_enum()
    }

    pub fn int8<I: Into<Secs2Int8Item>>(item: I) -> Secs2Variant {
        Secs2Int8::new(item.into()).as_enum()
    }

    pub fn list<I: Into<Secs2ListItem>>(item: I) -> Secs2Variant {
        Secs2List::new(item.into()).as_enum()
    }

    pub fn uint1<I: Into<Secs2Uint1Item>>(item: I) -> Secs2Variant {
        Secs2Uint1::new(item.into()).as_enum()
    }

    pub fn uint2<I: Into<Secs2Uint2Item>>(item: I) -> Secs2Variant {
        Secs2Uint2::new(item.into()).as_enum()
    }

    pub fn uint4<I: Into<Secs2Uint4Item>>(item: I) -> Secs2Variant {
        Secs2Uint4::new(item.into()).as_enum()
    }

    pub fn uint8<I: Into<Secs2Uint8Item>>(item: I) -> Secs2Variant {
        Secs2Uint8::new(item.into()).as_enum()
    }

    pub fn binary_unit(item: u8) -> Secs2Variant {
        Self::binary(vec![item])
    }

    pub fn boolean_unit(item: bool) -> Secs2Variant {
        Self::boolean(vec![item])
    }

    pub fn float4_unit(item: f32) -> Secs2Variant {
        Self::float4(vec![item])
    }

    pub fn float8_unit(item: f64) -> Secs2Variant {
        Self::float8(vec![item])
    }

    pub fn int1_unit(item: i8) -> Secs2Variant {
        Self::int1(vec![item])
    }

    pub fn int2_unit(item: i16) -> Secs2Variant {
        Self::int2(vec![item])
    }

    pub fn int4_unit(item: i32) -> Secs2Variant {
        Self::int4(vec![item])
    }

    pub fn int8_unit(item: i64) -> Secs2Variant {
        Self::int8(vec![item])
    }

    pub fn uint1_unit(item: u8) -> Secs2Variant {
        Self::uint1(vec![item])
    }

    pub fn uint2_unit(item: u16) -> Secs2Variant {
        Self::uint2(vec![item])
    }

    pub fn uint4_unit(item: u32) -> Secs2Variant {
        Self::uint4(vec![item])
    }

    pub fn uint8_unit(item: u64) -> Secs2Variant {
        Self::uint8(vec![item])
    }
}

impl Encode for Secs2Variant {
    fn encode(&self, w: &mut Vec<u8>) -> Result<(), Secs2Error> {
        use crate::convert::secs2;
        secs2::serialize::serialize_to(w, self)
    }
}

impl TryFrom<&[u8]> for Secs2Variant {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        use crate::convert::secs2;
        secs2::parse::parse(&value)
    }
}

/// SECS2 아이템 타입 코드를 표현하는 enum
#[derive(Debug, TryFromPrimitive, IntoPrimitive, PartialEq, Eq)]
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

pub trait Secs2Item {
    ///
    /// item을 enum으로 변환한다.
    ///
    fn as_enum(self) -> Secs2Variant;

    ///
    /// item의 길이를 반환한다
    ///
    fn length(&self) -> usize;
}

#[cfg(test)]
mod tests {
    use core::panic;

    use super::*;

    #[test]
    fn try_from_for_variant() {
        let data: Vec<u8> = vec![
            0x01, 0x02, 0x21, 0x02, 0x0B, 0x0C, 0x41, 0x05, 0x68, 0x65, 0x6C, 0x6C, 0x6F,
        ];

        let variant =
            Secs2Variant::try_from(data.as_slice()).expect("should ok <L> <B[2] 11, 12> <A hello>");
        let Secs2Variant::List(list) = &variant else {
            panic!("expected list variant");
        };

        let items = list.items();
        assert_eq!(items.len(), 2);

        // binary variant
        let binary_variant = &items[0];
        let Secs2Variant::Binary(bin) = binary_variant else {
            panic!("expected binary item, but found {:?}", binary_variant);
        };

        let bin_values = bin.items();
        assert_eq!(bin_values.len(), 2);
        assert_eq!(bin_values[0], 11);
        assert_eq!(bin_values[1], 12);

        // ascii variant
        let ascii_variant = &items[1];
        let Secs2Variant::ASCII(ascii) = ascii_variant else {
            panic!("expected ascii item, but found {:?}", ascii_variant);
        };

        let text = ascii.items();
        assert_eq!(text, "hello");
    }

    #[test]
    fn encode_for_variant() {
        let expected: Vec<u8> = vec![
            0x01, 0x02, 0x21, 0x02, 0x0B, 0x0C, 0x41, 0x05, 0x68, 0x65, 0x6C, 0x6C, 0x6F,
        ];

        let variant = Secs2Variant::list(vec![
            Secs2Variant::binary(vec![11u8, 12u8]),
            Secs2Variant::ascii(String::from("hello")),
        ]);

        let mut buf: Vec<u8> = Vec::new();
        variant.encode(&mut buf).expect("must be encoded");

        assert_eq!(buf, expected);
    }
}

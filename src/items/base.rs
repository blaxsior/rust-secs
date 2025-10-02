use crate::items::{
    ascii::Secs2ASCII, binary::Secs2Binary, boolean::Secs2Boolean, float4::Secs2Float4,
    float8::Secs2Float8, int1::Secs2Int1, int2::Secs2Int2, int4::Secs2Int4, int8::Secs2Int8,
    list::Secs2List, uint1::Secs2Uint1, uint2::Secs2Uint2, uint4::Secs2Uint4, uint8::Secs2Uint8,
};

/// u8 타입 
pub type Byte = u8;

/// Secs-II 규격에 맞는 아이템 요소를 의미하는 trait
pub trait Secs2Item {
    type ItemType;
    /// 내부 아이템을 반환
    fn items(&self) -> &Self::ItemType;
    /// 내부 아이템을 변경 가능하게 반환
    fn items_as_mut(&mut self) -> &mut Self::ItemType;
}

/// Secs2Item 객체를 표현하는 Enum
///
#[repr(u8)]
pub enum Secs2Type {
    LIST(Secs2List) = 0o00,
    BINARY(Secs2Binary) = 0o10,
    BOOLEAN(Secs2Boolean) = 0o11,
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

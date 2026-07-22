use crate::{
    convert::secs2::serialize::Encode,
    item::{Secs2Item, Secs2Variant},
};
use alloc::vec::Vec;

pub type Secs2Int2Item = Vec<i16>;
static SECS2_INT2_SIZE: usize = 2;
#[derive(Debug)]
pub struct Secs2Int2 {
    item: Secs2Int2Item,
}

impl Secs2Int2 {
    pub fn items(&self) -> &Secs2Int2Item {
        &self.item
    }

    pub fn items_as_mut(&mut self) -> &mut Secs2Int2Item {
        &mut self.item
    }

    pub fn new(item: Secs2Int2Item) -> Self {
        Self { item }
    }
}

impl Secs2Item for Secs2Int2 {
    fn as_enum(self) -> Secs2Variant {
        Secs2Variant::Int2(self)
    }

    fn length(&self) -> usize {
        self.item.len() * SECS2_INT2_SIZE
    }
}

impl Encode for Secs2Int2 {
    fn encode(&self, w: &mut Vec<u8>) -> Result<(), crate::error::Secs2Error> {
        for v in &self.item {
            w.extend_from_slice(&v.to_be_bytes());
        }

        Ok(())
    }
}

impl TryFrom<&[u8]> for Secs2Int2 {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if (value.len() % SECS2_INT2_SIZE) != 0 {
            return Err("input data size is invalid");
        }

        let result = value
            .chunks_exact(SECS2_INT2_SIZE)
            .map(|chunk| ((chunk[0] as i16) << 8) | (chunk[1] as i16))
            .collect();

        Ok(Secs2Int2::new(result))
    }
}

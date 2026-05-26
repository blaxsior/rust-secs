use crate::{
    convert::secs2::serialize::Encode,
    item::{Secs2Item, Secs2Variant},
};
use alloc::vec::Vec;

pub type Secs2Uint1Item = Vec<u8>;

// static SECS2_UINT1_SIZE: usize = 1;
#[derive(Debug)]
pub struct Secs2Uint1 {
    item: Secs2Uint1Item,
}

impl Secs2Uint1 {
    pub fn items(&self) -> &Secs2Uint1Item {
        &self.item
    }

    pub fn items_as_mut(&mut self) -> &mut Secs2Uint1Item {
        &mut self.item
    }

    pub fn new(item: Secs2Uint1Item) -> Self {
        Self { item }
    }
}

impl Secs2Item for Secs2Uint1 {
    fn as_enum(self) -> Secs2Variant {
        Secs2Variant::UInt1(self)
    }

    fn length(&self) -> usize {
        self.item.len()
    }
}

impl Encode for Secs2Uint1 {
    fn encode(&self, w: &mut Vec<u8>) -> Result<(), crate::error::Secs2Error> {
        for v in &self.item {
            w.extend_from_slice(&v.to_be_bytes());
        }

        Ok(())
    }
}

impl TryFrom<&[u8]> for Secs2Uint1 {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(Secs2Uint1::new(value.to_vec()))
    }
}

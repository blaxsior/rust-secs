use crate::item::{Secs2Variant, Secs2Item};

type Secs2Uint1Item = Vec<u8>;

// static SECS2_UINT1_SIZE: usize = 1;

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

impl TryFrom<&[u8]> for Secs2Uint1 {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(Secs2Uint1::new(value.to_vec()))
    }
}

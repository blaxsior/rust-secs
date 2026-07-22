use crate::{
    convert::secs2::serialize::Encode,
    item::{Secs2Item, Secs2Variant},
};
use alloc::string::String;
use alloc::vec::Vec;

pub type Secs2ASCIIItem = String;
#[derive(Debug)]
pub struct Secs2ASCII {
    item: Secs2ASCIIItem,
}

impl Secs2ASCII {
    pub fn items(&self) -> &Secs2ASCIIItem {
        &self.item
    }

    pub fn items_as_mut(&mut self) -> &mut Secs2ASCIIItem {
        &mut self.item
    }

    pub fn new(item: Secs2ASCIIItem) -> Self {
        Self { item }
    }
}

impl Secs2Item for Secs2ASCII {
    fn as_enum(self) -> Secs2Variant {
        Secs2Variant::ASCII(self)
    }

    fn length(&self) -> usize {
        self.item.chars().count()
    }
}

impl Encode for Secs2ASCII {
    fn encode(&self, w: &mut Vec<u8>) -> Result<(), crate::error::Secs2Error> {
        w.extend_from_slice(self.item.as_bytes());
        Ok(())
    }
}

impl TryFrom<&[u8]> for Secs2ASCII {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let result =
            String::from_utf8(value.to_vec()).expect("failed to parse buffer to ascii string");

        if result.is_ascii() {
            Ok(Self::new(result))
        } else {
            Err("result data is not ascii string")
        }
    }
}

use crate::items::base::{Secs2Item, Secs2ItemType};

type Secs2Uint8Value = Vec<u64>;
pub struct Secs2Uint8 {
    item: Secs2Uint8Value,
}

impl Secs2Uint8 {
    fn items(&self) -> & Secs2Uint8Value {
        &self.item
    }

    fn items_as_mut(&mut self) -> &mut Secs2Uint8Value {
        &mut self.item
    }

        fn new(item: Secs2Uint8Value) -> Self {
        Self { item }
    }
}

impl Secs2Item for Secs2Uint8 {
    fn as_enum(self) -> Secs2ItemType {
        Secs2ItemType::UInt8(self)
    }
    
    fn item_length(&self) -> usize {
        self.item.len() * 8
    }
}

impl ToString for Secs2Uint8 {
    fn to_string(&self) -> String {
        todo!()
    }
}
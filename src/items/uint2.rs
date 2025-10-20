use crate::items::base::{Secs2Item, Secs2ItemType};

type Secs2Uint2Value = Vec<u16>;
pub struct Secs2Uint2 {
    item: Secs2Uint2Value,
}

impl Secs2Uint2 {
    fn items(&self) -> &Secs2Uint2Value {
        &self.item
    }

    fn items_as_mut(&mut self) -> &mut Secs2Uint2Value {
        &mut self.item
    }

    fn new(item: Secs2Uint2Value) -> Self {
        Self { item }
    }
}

impl Secs2Item for Secs2Uint2 {
    fn as_enum(self) -> Secs2ItemType {
        Secs2ItemType::UInt2(self)
    }
    
    fn item_length(&self) -> usize {
        self.item.len() * 2
    }
}

impl ToString for Secs2Uint2 {
    fn to_string(&self) -> String {
        todo!()
    }
}

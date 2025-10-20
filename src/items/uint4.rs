use crate::items::base::{Secs2Item, Secs2ItemType};

type Secs2Uint4Value = Vec<u32>;
pub struct Secs2Uint4 {
    item: Secs2Uint4Value,
}

impl Secs2Uint4 {

    fn items(&self) -> &Secs2Uint4Value {
        &self.item
    }

    fn items_as_mut(&mut self) -> &mut Secs2Uint4Value {
        &mut self.item
    }

    fn new(item: Secs2Uint4Value) -> Self {
        Self { item }
    }
}

impl Secs2Item for Secs2Uint4 {
    fn as_enum(self) -> Secs2ItemType {
        Secs2ItemType::UInt4(self)
    }
    
    fn item_length(&self) -> usize {
        self.item.len() * 4
    }
}

impl ToString for Secs2Uint4 {
    fn to_string(&self) -> String {
        todo!()
    }
}
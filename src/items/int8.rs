use crate::items::base::{Secs2Item, Secs2ItemType};

type Secs2Int8Value = Vec<i64>;
pub struct Secs2Int8 {
    item: Secs2Int8Value,
}

impl Secs2Int8 {
    fn items(&self) -> &Secs2Int8Value {
        &self.item
    }

    fn items_as_mut(&mut self) -> &mut Secs2Int8Value {
        &mut self.item
    }

     fn new(item: Secs2Int8Value) -> Self {
        Self { item }
    }
}


impl Secs2Item for Secs2Int8 {
    fn as_enum(self) -> super::base::Secs2ItemType {
        Secs2ItemType::Int8(self)
    }
    
    fn item_length(&self) -> usize {
        self.item.len() * 8
    }
}

impl ToString for Secs2Int8 {
    fn to_string(&self) -> String {
        todo!()
    }
}

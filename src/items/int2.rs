use crate::items::base::{Secs2Item, Secs2ItemType};

type Secs2Int2Value = Vec<i16>;
pub struct Secs2Int2 {
    item: Secs2Int2Value,
}

impl Secs2Int2 {
    fn items(&self) -> &Secs2Int2Value {
        &self.item
    }

    fn items_as_mut(&mut self) -> &mut Secs2Int2Value {
        &mut self.item
    }

    fn new(item: Secs2Int2Value) -> Self {
        Self { item }
    }
}

impl Secs2Item for Secs2Int2 {
    fn as_enum(self) -> Secs2ItemType {
        Secs2ItemType::Int2(self)
    }
    
    fn item_length(&self) -> usize {
        self.item.len() * 2
    }
}

impl ToString for Secs2Int2 {
    fn to_string(&self) -> String {
        todo!()
    }
}

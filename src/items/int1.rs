use crate::items::base::{Secs2Item, Secs2ItemType};

type Secs2Int1Value = Vec<i8>;
pub struct Secs2Int1 {
    item: Secs2Int1Value,
}

impl Secs2Int1 {
    fn items(&self) -> &Secs2Int1Value {
        &self.item
    }

    fn items_as_mut(&mut self) -> &mut Secs2Int1Value {
        &mut self.item
    }

    fn new(item: Secs2Int1Value) -> Self {
        Self { item }
    }
}

impl Secs2Item for Secs2Int1 {
    fn as_enum(self) -> Secs2ItemType {
        Secs2ItemType::Int1(self)
    }

    fn item_length(&self) -> usize {
        self.item.len()
    }
}

impl ToString for Secs2Int1 {
    fn to_string(&self) -> String {
        todo!()
    }
}

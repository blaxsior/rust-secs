use crate::items::base::{Secs2Item, Secs2ItemType};

type Secs2Int4Value = Vec<i32>;
pub struct Secs2Int4 {
    item: Secs2Int4Value,
}

impl Secs2Int4 {
    fn items(&self) -> &Secs2Int4Value {
        &self.item
    }

    fn items_as_mut(&mut self) -> &mut Secs2Int4Value {
        &mut self.item
    }

    fn new(item: Secs2Int4Value) -> Self {
        Self { item }
    }
}

impl Secs2Item for Secs2Int4 {
    fn as_enum(self) -> Secs2ItemType {
        Secs2ItemType::Int4(self)
    }

    fn item_length(&self) -> usize {
        self.item.len() * 4
    }
}

impl ToString for Secs2Int4 {
    fn to_string(&self) -> String {
        todo!()
    }
}

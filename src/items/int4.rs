use crate::items::base::Secs2Item;

pub struct Secs2Int4 {
    items: Vec<i32>,
}

impl Secs2Item for Secs2Int4 {
    type ItemType = Vec<i32>;

    fn items(&self) -> &Self::ItemType {
        &self.items
    }

    fn items_as_mut(&mut self) -> &mut Self::ItemType {
        &mut self.items
    }
}

use crate::items::base::Secs2Item;

pub struct Secs2Int8 {
    items: Vec<i64>,
}

impl Secs2Item for Secs2Int8 {
    type ItemType = Vec<i64>;

    fn items(&self) -> &Self::ItemType {
        &self.items
    }

    fn items_as_mut(&mut self) -> &mut Self::ItemType {
        &mut self.items
    }
}

use crate::items::base::Secs2Item;

pub struct Secs2Int1 {
    items: Vec<i8>,
}

impl Secs2Item for Secs2Int1 {
    type ItemType = Vec<i8>;

    fn items(&self) -> &Self::ItemType {
        &self.items
    }

    fn items_as_mut(&mut self) -> &mut Self::ItemType {
        &mut self.items
    }
}

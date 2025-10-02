use crate::items::base::{Secs2Item, Secs2Type};

pub struct Secs2List {
    items: Vec<Box<dyn Secs2Item<ItemType = Secs2Type>>>,
}

impl Secs2Item for Secs2List {
    type ItemType = Vec<Box<dyn Secs2Item<ItemType = Secs2Type>>>;

    fn items(&self) -> &Self::ItemType {
        &self.items
    }

    fn items_as_mut(&mut self) -> &mut Self::ItemType {
        &mut self.items
    }
}

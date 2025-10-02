use crate::items::base::{Byte, Secs2Item};

pub struct Secs2Boolean {
    items: Vec<Byte>,
}

impl Secs2Item for Secs2Boolean {
    type ItemType = Vec<Byte>;

    fn items(&self) -> &Self::ItemType {
        &self.items
    }

    fn items_as_mut(&mut self) -> &mut Self::ItemType {
        &mut self.items
    }
}

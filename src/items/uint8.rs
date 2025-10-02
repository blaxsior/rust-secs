use crate::items::base::Secs2Item;

pub struct Secs2Uint8 {
    items: Vec<u64>,
}

impl Secs2Item for Secs2Uint8 {
    type ItemType = Vec<u64>;

    fn items(&self) -> &Self::ItemType {
        &self.items
    }

    fn items_as_mut(&mut self) -> &mut Self::ItemType {
        &mut self.items
    }
}

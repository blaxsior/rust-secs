use crate::items::base::Secs2Item;

pub struct Secs2Uint4 {
    items: Vec<u32>,
}

impl Secs2Item for Secs2Uint4 {
    type ItemType = Vec<u32>;

    fn items(&self) -> &Self::ItemType {
        &self.items
    }

    fn items_as_mut(&mut self) -> &mut Self::ItemType {
        &mut self.items
    }
}

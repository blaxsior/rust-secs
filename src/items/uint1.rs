use crate::items::base::Secs2Item;

pub struct Secs2Uint1 {
    items: Vec<u8>,
}

impl Secs2Item for Secs2Uint1 {
    type ItemType = Vec<u8>;

    fn items(&self) -> &Self::ItemType {
        &self.items
    }

    fn items_as_mut(&mut self) -> &mut Self::ItemType {
        &mut self.items
    }
}

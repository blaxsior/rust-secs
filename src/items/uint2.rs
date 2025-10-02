use crate::items::base::Secs2Item;

pub struct Secs2Uint2 {
    items: Vec<u16>,
}

impl Secs2Item for Secs2Uint2 {
    type ItemType = Vec<u16>;

    fn items(&self) -> &Self::ItemType {
        &self.items
    }

    fn items_as_mut(&mut self) -> &mut Self::ItemType {
        &mut self.items
    }
}

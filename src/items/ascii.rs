use crate::items::base::Secs2Item;

pub struct Secs2ASCII {
    items: String,
}

impl Secs2Item for Secs2ASCII {
    type ItemType = String;

    fn items(&self) -> &Self::ItemType {
        &self.items
    }

    fn items_as_mut(&mut self) -> &mut Self::ItemType {
        &mut self.items
    }
}

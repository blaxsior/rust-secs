use crate::items::base::Secs2Item;

pub struct Secs2Int2 {
    items: Vec<i16>,
}

impl Secs2Item for Secs2Int2 {
    type ItemType = Vec<i16>;

    fn items(&self) -> &Self::ItemType {
        &self.items
    }

    fn items_as_mut(&mut self) -> &mut Self::ItemType {
        &mut self.items
    }
}

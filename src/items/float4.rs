use crate::items::base::Secs2Item;

pub struct Secs2Float4 {
    items: Vec<f32>,
}

impl Secs2Item for Secs2Float4 {
    type ItemType = Vec<f32>;

    fn items(&self) -> &Self::ItemType {
        &self.items
    }

    fn items_as_mut(&mut self) -> &mut Self::ItemType {
        &mut self.items
    }
}

use crate::items::base::Secs2Item;

pub struct Secs2Float8 {
    items: Vec<f64>,
}

impl Secs2Item for Secs2Float8 {
    type ItemType = Vec<f64>;

    fn items(&self) -> &Self::ItemType {
        &self.items
    }

    fn items_as_mut(&mut self) -> &mut Self::ItemType {
        &mut self.items
    }
}

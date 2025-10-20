use crate::items::base::{Secs2Item, Secs2ItemType};

type Secs2Float8Value = Vec<f64>;
pub struct Secs2Float8 {
    item: Secs2Float8Value,
}

impl Secs2Float8 {
    fn items(&self) -> &Secs2Float8Value {
        &self.item
    }

    fn items_as_mut(&mut self) -> &mut Secs2Float8Value {
        &mut self.item
    }

    fn new(item: Secs2Float8Value) -> Self {
        Self { item }
    }
}

impl Secs2Item for Secs2Float8 {
    fn as_enum(self) -> Secs2ItemType {
        Secs2ItemType::Float8(self)
    }

    fn item_length(&self) -> usize {
        self.item.len() * 8
    }
}

impl ToString for Secs2Float8 {
    fn to_string(&self) -> String {
        todo!()
    }
}

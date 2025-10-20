use crate::items::base::{Secs2Item, Secs2ItemType};

type Secs2Float4Value = Vec<f32>;
pub struct Secs2Float4 {
    item: Secs2Float4Value,
}

impl Secs2Float4 {
    fn items(&self) -> &Secs2Float4Value {
        &self.item
    }

    fn items_as_mut(&mut self) -> &mut Secs2Float4Value {
        &mut self.item
    }

    fn new(item: Secs2Float4Value) -> Self {
        Self { item }
    }
}

impl Secs2Item for Secs2Float4 {
    fn as_enum(self) -> Secs2ItemType {
        Secs2ItemType::Float4(self)
    }
    
    fn item_length(&self) -> usize {
        self.item.len() * 4
    }
}

impl ToString for Secs2Float4 {
    fn to_string(&self) -> String {
        todo!()
    }
}

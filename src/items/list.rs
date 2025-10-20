use crate::items::base::{Secs2Item, Secs2ItemType};

type Secs2ListValue = Vec<Box<dyn Secs2Item>>;

pub struct Secs2List {
    item: Secs2ListValue,
}

impl Secs2List {
    pub fn items(&self) -> &Secs2ListValue {
        &self.item
    }

    pub fn items_as_mut(&mut self) -> &mut Secs2ListValue {
        &mut self.item
    }

    pub fn new(item: Secs2ListValue) -> Self {
        Self { item }
    }
}

impl Secs2Item for Secs2List {
    fn as_enum(self) -> Secs2ItemType {
        return Secs2ItemType::List(self);
    }

    fn item_length(&self) -> usize {
        return self.item.len()
    }
}

impl ToString for Secs2List {
    fn to_string(&self) -> String {
        todo!()
    }
}

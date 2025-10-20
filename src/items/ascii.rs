use crate::items::base::{Secs2Item, Secs2ItemType};

type Secs2ASCIIValue = String;
pub struct Secs2ASCII {
    item: Secs2ASCIIValue,
}

impl Secs2ASCII {
    fn items(&self) -> &Secs2ASCIIValue {
        &self.item
    }

    fn items_as_mut(&mut self) -> &mut Secs2ASCIIValue {
        &mut self.item
    }

    fn new(item: Secs2ASCIIValue) -> Self {
        Self { item }
    }
}

impl Secs2Item for Secs2ASCII {
    fn as_enum(self) -> Secs2ItemType {
        Secs2ItemType::ASCII(self)
    }

    fn item_length(&self) -> usize {
        self.item.chars().count()
    }
}

impl ToString for Secs2ASCII {
    fn to_string(&self) -> String {
        todo!()
    }
}

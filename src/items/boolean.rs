use crate::items::base::{Secs2Item, Secs2ItemCode, Secs2ItemType};

type Secs2BooleanValue = Vec<u8>;
pub struct Secs2Boolean {
    item: Secs2BooleanValue,
}

impl Secs2Boolean {
    fn items(&self) -> &Secs2BooleanValue {
        &self.item
    }

    fn items_as_mut(&mut self) -> &mut Secs2BooleanValue {
        &mut self.item
    }

    fn new(item: Secs2BooleanValue) -> Self {
        Self { item }
    }
}

impl Secs2Item for Secs2Boolean {
    fn as_enum(self) -> Secs2ItemType {
        Secs2ItemType::Boolean(self)
    }

    fn item_length(&self) -> usize {
        self.item.len()
    }
    
    fn item_code() -> super::base::Secs2ItemCode {
        Secs2ItemCode::Boolean
    }
}

impl ToString for Secs2Boolean {
    fn to_string(&self) -> String {
        todo!()
    }
}

impl TryFrom<Vec<u8>> for Secs2Boolean {
    type Error = &'static str;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(Secs2Boolean::new(value))
    }
}

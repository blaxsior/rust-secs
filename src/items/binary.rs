use crate::items::base::{Secs2Item, Secs2ItemType};

type Secs2BinaryValue = Vec<u8>;
pub struct Secs2Binary {
    item: Secs2BinaryValue,
}

impl Secs2Binary {
    fn items(&self) -> &Secs2BinaryValue {
        &self.item
    }

    fn items_as_mut(&mut self) -> &mut Secs2BinaryValue {
        &mut self.item
    }

    fn new(item: Secs2BinaryValue) -> Self {
        Self { item }
    }
}

impl Secs2Item for Secs2Binary {
    fn as_enum(self) -> Secs2ItemType {
        Secs2ItemType::Binary(self)
    }

    fn item_length(&self) -> usize {
        self.item.len()
    }
}

impl ToString for Secs2Binary {
    fn to_string(&self) -> String {
        todo!()
    }
}

impl TryFrom<Vec<u8>> for Secs2Binary {
    type Error = &'static str;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(Secs2Binary::new(value))
    }
}

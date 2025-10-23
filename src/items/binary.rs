use crate::items::base::{Secs2ItemBody, Secs2Item};

type Secs2BinaryItem = Vec<u8>;
pub struct Secs2BinaryBody {
    item: Secs2BinaryItem,
}

impl Secs2BinaryBody {
    fn items(&self) -> &Secs2BinaryItem {
        &self.item
    }

    fn items_as_mut(&mut self) -> &mut Secs2BinaryItem {
        &mut self.item
    }

    fn new(item: Secs2BinaryItem) -> Self {
        Self { item }
    }
}

impl Secs2ItemBody for Secs2BinaryBody {
    fn as_enum(self) -> Secs2Item {
        Secs2Item::Binary(self)
    }

    fn item_length(&self) -> usize {
        self.item.len()
    }
}

impl ToString for Secs2BinaryBody {
    fn to_string(&self) -> String {
        todo!()
    }
}

impl TryFrom<&[u8]> for Secs2BinaryBody {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(Secs2BinaryBody::new(value.to_vec()))
    }
}
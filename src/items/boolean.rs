use crate::items::base::{Secs2ItemBody, Secs2Item};

type Secs2BooleanItem = Vec<u8>;
pub struct Secs2BooleanBody {
    item: Secs2BooleanItem,
}

impl Secs2BooleanBody {
    fn items(&self) -> &Secs2BooleanItem {
        &self.item
    }

    fn items_as_mut(&mut self) -> &mut Secs2BooleanItem {
        &mut self.item
    }

    fn new(item: Secs2BooleanItem) -> Self {
        Self { item }
    }
}

impl Secs2ItemBody for Secs2BooleanBody {
    fn as_enum(self) -> Secs2Item {
        Secs2Item::Boolean(self)
    }

    fn item_length(&self) -> usize {
        self.item.len()
    }
}

impl ToString for Secs2BooleanBody {
    fn to_string(&self) -> String {
        todo!()
    }
}

impl TryFrom<&[u8]> for Secs2BooleanBody {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(Secs2BooleanBody::new(value.to_vec()))
    }
}

use crate::items::base::{Secs2Item, Secs2ItemCode, Secs2ItemType};

type Secs2Uint1Value = Vec<u8>;

static SECS2_UINT1_SIZE: usize = 1;
pub struct Secs2Uint1 {
    item: Secs2Uint1Value,
}

impl Secs2Uint1 {
    fn items(&self) -> &Secs2Uint1Value {
        &self.item
    }

    fn items_as_mut(&mut self) -> &mut Secs2Uint1Value {
        &mut self.item
    }

    fn new(item: Secs2Uint1Value) -> Self {
        Self { item }
    }
}

impl Secs2Item for Secs2Uint1 {
    fn as_enum(self) -> Secs2ItemType {
        Secs2ItemType::UInt1(self)
    }

    fn item_length(&self) -> usize {
        self.item.len()
    }
    
    fn item_code() -> super::base::Secs2ItemCode {
        Secs2ItemCode::UInt1
    }
}

impl ToString for Secs2Uint1 {
    fn to_string(&self) -> String {
        todo!()
    }
}

impl TryFrom<Vec<u8>> for Secs2Uint1 {
    type Error = &'static str;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(Secs2Uint1::new(value))
    }
}

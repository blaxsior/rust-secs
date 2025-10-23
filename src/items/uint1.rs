use crate::items::base::{Secs2ItemBody, Secs2Item};

type Secs2Uint1Item = Vec<u8>;

static SECS2_UINT1_SIZE: usize = 1;
pub struct Secs2Uint1Body {
    item: Secs2Uint1Item,
}

impl Secs2Uint1Body {
    fn items(&self) -> &Secs2Uint1Item {
        &self.item
    }

    fn items_as_mut(&mut self) -> &mut Secs2Uint1Item {
        &mut self.item
    }

    fn new(item: Secs2Uint1Item) -> Self {
        Self { item }
    }
}

impl Secs2ItemBody for Secs2Uint1Body {
    fn as_enum(self) -> Secs2Item {
        Secs2Item::UInt1(self)
    }

    fn item_length(&self) -> usize {
        self.item.len()
    }
}

impl ToString for Secs2Uint1Body {
    fn to_string(&self) -> String {
        todo!()
    }
}

impl TryFrom<&[u8]> for Secs2Uint1Body {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(Secs2Uint1Body::new(value.to_vec()))
    }
}

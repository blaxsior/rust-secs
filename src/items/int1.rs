use crate::items::base::{Secs2ItemBody, Secs2Item};

type Secs2Int1Item = Vec<i8>;
static SECS2_INT1_SIZE: usize = 1;
pub struct Secs2Int1Body {
    item: Secs2Int1Item,
}

impl Secs2Int1Body {
    fn items(&self) -> &Secs2Int1Item {
        &self.item
    }

    fn items_as_mut(&mut self) -> &mut Secs2Int1Item {
        &mut self.item
    }

    fn new(item: Secs2Int1Item) -> Self {
        Self { item }
    }
}

impl Secs2ItemBody for Secs2Int1Body {
    fn as_enum(self) -> Secs2Item {
        Secs2Item::Int1(self)
    }

    fn item_length(&self) -> usize {
        self.item.len()
    }
}

impl ToString for Secs2Int1Body {
    fn to_string(&self) -> String {
        todo!()
    }
}

impl TryFrom<&[u8]> for Secs2Int1Body {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if (value.len() % SECS2_INT1_SIZE) != 0 {
            return Err("input data size is invalid");
        }

        let result = value.into_iter().map(|chunk| *chunk as i8).collect();

        Ok(Secs2Int1Body::new(result))
    }
}

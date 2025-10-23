use crate::items::base::{Secs2ItemBody, Secs2Item};

type Secs2Int2Item = Vec<i16>;
static SECS2_INT2_SIZE: usize = 2;

pub struct Secs2Int2Body {
    item: Secs2Int2Item,
}

impl Secs2Int2Body {
    fn items(&self) -> &Secs2Int2Item {
        &self.item
    }

    fn items_as_mut(&mut self) -> &mut Secs2Int2Item {
        &mut self.item
    }

    fn new(item: Secs2Int2Item) -> Self {
        Self { item }
    }
}

impl Secs2ItemBody for Secs2Int2Body {
    fn as_enum(self) -> Secs2Item {
        Secs2Item::Int2(self)
    }

    fn item_length(&self) -> usize {
        self.item.len() * SECS2_INT2_SIZE
    }
}

impl ToString for Secs2Int2Body {
    fn to_string(&self) -> String {
        todo!()
    }
}

impl TryFrom<&[u8]> for Secs2Int2Body {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if (value.len() % SECS2_INT2_SIZE) != 0 {
            return Err("input data size is invalid");
        }

        let result = value
            .chunks_exact(SECS2_INT2_SIZE)
            .map(|chunk| {
                let arr: [u8; SECS2_INT2_SIZE] =
                    chunk.try_into().expect("failed to convert bytes to F8");
                i16::from_be_bytes(arr)
            })
            .collect();

        Ok(Secs2Int2Body::new(result))
    }
}

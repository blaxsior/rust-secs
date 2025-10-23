use crate::items::base::{Secs2ItemBody, Secs2Item};

type Secs2Int8Item = Vec<i64>;
static SECS2_INT8_SIZE: usize = 8;

pub struct Secs2Int8Body {
    item: Secs2Int8Item,
}

impl Secs2Int8Body {
    fn items(&self) -> &Secs2Int8Item {
        &self.item
    }

    fn items_as_mut(&mut self) -> &mut Secs2Int8Item {
        &mut self.item
    }

     fn new(item: Secs2Int8Item) -> Self {
        Self { item }
    }
}


impl Secs2ItemBody for Secs2Int8Body {
    fn as_enum(self) -> super::base::Secs2Item {
        Secs2Item::Int8(self)
    }
    
    fn item_length(&self) -> usize {
        self.item.len() * SECS2_INT8_SIZE
    }
}

impl ToString for Secs2Int8Body {
    fn to_string(&self) -> String {
        todo!()
    }
}

impl TryFrom<&[u8]> for Secs2Int8Body {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if (value.len() % SECS2_INT8_SIZE) != 0 {
            return Err("input data size is invalid");
        }

        let result = value
            .chunks_exact(SECS2_INT8_SIZE)
            .map(|chunk| {
                let arr: [u8; SECS2_INT8_SIZE] =
                    chunk.try_into().expect("failed to convert bytes to F8");
                i64::from_be_bytes(arr)
            })
            .collect();

        Ok(Secs2Int8Body::new(result))
    }
}

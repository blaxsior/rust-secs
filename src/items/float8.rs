use crate::items::base::{Secs2ItemBody, Secs2Item};

type Secs2Float8Item = Vec<f64>;
static SECS2_FLOAT8_SIZE: usize = 8;
pub struct Secs2Float8Body {
    item: Secs2Float8Item,
}

impl Secs2Float8Body {
    fn items(&self) -> &Secs2Float8Item {
        &self.item
    }

    fn items_as_mut(&mut self) -> &mut Secs2Float8Item {
        &mut self.item
    }

    fn new(item: Secs2Float8Item) -> Self {
        Self { item }
    }
}

impl Secs2ItemBody for Secs2Float8Body {
    fn as_enum(self) -> Secs2Item {
        Secs2Item::Float8(self)
    }

    fn item_length(&self) -> usize {
        self.item.len() * SECS2_FLOAT8_SIZE
    }
}

impl ToString for Secs2Float8Body {
    fn to_string(&self) -> String {
        todo!()
    }
}

impl TryFrom<&[u8]> for Secs2Float8Body {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if (value.len() % SECS2_FLOAT8_SIZE) != 0 {
            return Err("input data size is invalid");
        }

        let result = value.chunks_exact(SECS2_FLOAT8_SIZE).map(|chunk| {
            let arr: [u8; SECS2_FLOAT8_SIZE] = chunk.try_into().expect("failed to convert bytes to F8");
            f64::from_be_bytes(arr)
        }).collect();

        Ok(Secs2Float8Body::new(result))
    }
}

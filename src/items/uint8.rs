use crate::items::base::{Secs2ItemBody, Secs2Item};

type Secs2Uint8Item = Vec<u64>;
static SECS2_UINT8_SIZE: usize = 8;
pub struct Secs2Uint8Body {
    item: Secs2Uint8Item,
}

impl Secs2Uint8Body {
    fn items(&self) -> & Secs2Uint8Item {
        &self.item
    }

    fn items_as_mut(&mut self) -> &mut Secs2Uint8Item {
        &mut self.item
    }

        fn new(item: Secs2Uint8Item) -> Self {
        Self { item }
    }
}

impl Secs2ItemBody for Secs2Uint8Body {
    fn as_enum(self) -> Secs2Item {
        Secs2Item::UInt8(self)
    }
    
    fn item_length(&self) -> usize {
        self.item.len() * SECS2_UINT8_SIZE
    }
}

impl ToString for Secs2Uint8Body {
    fn to_string(&self) -> String {
        todo!()
    }
}

impl TryFrom<&[u8]> for Secs2Uint8Body {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if (value.len() % SECS2_UINT8_SIZE) != 0 {
            return Err("input data size is invalid");
        }

        let result = value
            .chunks_exact(SECS2_UINT8_SIZE)
            .map(|chunk| {
                let arr: [u8; SECS2_UINT8_SIZE] =
                    chunk.try_into().expect("failed to convert bytes to F8");
                u64::from_be_bytes(arr)
            })
            .collect();

        Ok(Secs2Uint8Body::new(result))
    }
}
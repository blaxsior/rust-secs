use crate::items::base::{Secs2ItemBody, Secs2Item};

type Secs2Uint2Item = Vec<u16>;
static SECS2_UINT2_SIZE: usize = 2;
pub struct Secs2Uint2Body {
    item: Secs2Uint2Item,
}

impl Secs2Uint2Body {
    fn items(&self) -> &Secs2Uint2Item {
        &self.item
    }

    fn items_as_mut(&mut self) -> &mut Secs2Uint2Item {
        &mut self.item
    }

    fn new(item: Secs2Uint2Item) -> Self {
        Self { item }
    }
}

impl Secs2ItemBody for Secs2Uint2Body {
    fn as_enum(self) -> Secs2Item {
        Secs2Item::UInt2(self)
    }

    fn item_length(&self) -> usize {
        self.item.len() * SECS2_UINT2_SIZE
    }
}

impl ToString for Secs2Uint2Body {
    fn to_string(&self) -> String {
        todo!()
    }
}

impl TryFrom<&[u8]> for Secs2Uint2Body {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if (value.len() % SECS2_UINT2_SIZE) != 0 {
            return Err("input data size is invalid");
        }

        let result = value
            .chunks_exact(SECS2_UINT2_SIZE)
            .map(|chunk| {
                let arr: [u8; SECS2_UINT2_SIZE] =
                    chunk.try_into().expect("failed to convert bytes to F8");
                u16::from_be_bytes(arr)
            })
            .collect();

        Ok(Secs2Uint2Body::new(result))
    }
}

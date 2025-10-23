use crate::items::base::{Secs2ItemBody, Secs2Item};

type Secs2Uint4Item = Vec<u32>;
static SECS2_UINT4_SIZE: usize = 4;
pub struct Secs2Uint4Body {
    item: Secs2Uint4Item,
}

impl Secs2Uint4Body {
    fn items(&self) -> &Secs2Uint4Item {
        &self.item
    }

    fn items_as_mut(&mut self) -> &mut Secs2Uint4Item {
        &mut self.item
    }

    fn new(item: Secs2Uint4Item) -> Self {
        Self { item }
    }
}

impl Secs2ItemBody for Secs2Uint4Body {
    fn as_enum(self) -> Secs2Item {
        Secs2Item::UInt4(self)
    }

    fn item_length(&self) -> usize {
        self.item.len() * SECS2_UINT4_SIZE
    }
}

impl ToString for Secs2Uint4Body {
    fn to_string(&self) -> String {
        todo!()
    }
}

impl TryFrom<&[u8]> for Secs2Uint4Body {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if (value.len() % SECS2_UINT4_SIZE) != 0 {
            return Err("input data size is invalid");
        }

        let result = value
            .chunks_exact(SECS2_UINT4_SIZE)
            .map(|chunk| {
                let arr: [u8; SECS2_UINT4_SIZE] =
                    chunk.try_into().expect("failed to convert bytes to F8");
                u32::from_be_bytes(arr)
            })
            .collect();

        Ok(Secs2Uint4Body::new(result))
    }
}

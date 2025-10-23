use crate::items::base::{Secs2ItemBody, Secs2Item};

type Secs2Int4Item = Vec<i32>;
static SECS4_INT4_SIZE: usize = 4;
pub struct Secs2Int4Body {
    item: Secs2Int4Item,
}

impl Secs2Int4Body {
    fn items(&self) -> &Secs2Int4Item {
        &self.item
    }

    fn items_as_mut(&mut self) -> &mut Secs2Int4Item {
        &mut self.item
    }

    fn new(item: Secs2Int4Item) -> Self {
        Self { item }
    }
}

impl Secs2ItemBody for Secs2Int4Body {
    fn as_enum(self) -> Secs2Item {
        Secs2Item::Int4(self)
    }

    fn item_length(&self) -> usize {
        self.item.len() * SECS4_INT4_SIZE
    }
}

impl ToString for Secs2Int4Body {
    fn to_string(&self) -> String {
        todo!()
    }
}

impl TryFrom<&[u8]> for Secs2Int4Body {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if (value.len() % SECS4_INT4_SIZE) != 0 {
            return Err("input data size is invalid");
        }

        let result = value
            .chunks_exact(SECS4_INT4_SIZE)
            .map(|chunk| {
                let arr: [u8; SECS4_INT4_SIZE] =
                    chunk.try_into().expect("failed to convert bytes to F8");
                i32::from_be_bytes(arr)
            })
            .collect();

        Ok(Secs2Int4Body::new(result))
    }
}

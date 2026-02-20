use crate::item::{Secs2Item, Secs2Variant};

type Secs2Int4Item = Vec<i32>;
static SECS2_INT4_SIZE: usize = 4;
pub struct Secs2Int4 {
    item: Secs2Int4Item,
}

impl Secs2Int4 {
    pub fn items(&self) -> &Secs2Int4Item {
        &self.item
    }

    pub fn items_as_mut(&mut self) -> &mut Secs2Int4Item {
        &mut self.item
    }

    pub fn new(item: Secs2Int4Item) -> Self {
        Self { item }
    }
}

impl Secs2Item for Secs2Int4 {
    fn as_enum(self) -> Secs2Variant {
        Secs2Variant::Int4(self)
    }

    fn length(&self) -> usize {
        self.item.len() * SECS2_INT4_SIZE
    }
}

impl TryFrom<&[u8]> for Secs2Int4 {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if (value.len() % SECS2_INT4_SIZE) != 0 {
            return Err("input data size is invalid");
        }

        let result = value
            .chunks_exact(SECS2_INT4_SIZE)
            .map(|chunk| {
                let mut val = 0i32;
                for i in 0..SECS2_INT4_SIZE {
                    val <<= 8;
                    val |= chunk[i] as i32;
                }
                val
            })
            .collect();

        Ok(Secs2Int4::new(result))
    }
}

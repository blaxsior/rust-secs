use crate::item::{Secs2Item, Secs2Variant};

type Secs2Int8Item = Vec<i64>;
static SECS2_INT8_SIZE: usize = 8;
#[derive(Debug)]
pub struct Secs2Int8 {
    item: Secs2Int8Item,
}

impl Secs2Int8 {
    pub fn items(&self) -> &Secs2Int8Item {
        &self.item
    }

    pub fn items_as_mut(&mut self) -> &mut Secs2Int8Item {
        &mut self.item
    }

    pub fn new(item: Secs2Int8Item) -> Self {
        Self { item }
    }
}

impl Secs2Item for Secs2Int8 {
    fn as_enum(self) -> Secs2Variant {
        Secs2Variant::Int8(self)
    }

    fn length(&self) -> usize {
        self.item.len() * SECS2_INT8_SIZE
    }
}

impl TryFrom<&[u8]> for Secs2Int8 {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if (value.len() % SECS2_INT8_SIZE) != 0 {
            return Err("input data size is invalid");
        }

        let result = value
            .chunks_exact(SECS2_INT8_SIZE)
            .map(|chunk| {
                let mut val = 0i64;
                for i in 0..SECS2_INT8_SIZE {
                    val <<= 8;
                    val |= chunk[i] as i64; 
                }
                val
            })
            .collect();

        Ok(Secs2Int8::new(result))
    }
}

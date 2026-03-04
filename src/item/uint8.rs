use crate::item::{Secs2Item, Secs2Variant};

type Secs2Uint8Item = Vec<u64>;
static SECS2_UINT8_SIZE: usize = 8;
#[derive(Debug)]
pub struct Secs2Uint8 {
    item: Secs2Uint8Item,
}

impl Secs2Uint8 {
    pub fn items(&self) -> &Secs2Uint8Item {
        &self.item
    }

    pub fn items_as_mut(&mut self) -> &mut Secs2Uint8Item {
        &mut self.item
    }

    pub fn new(item: Secs2Uint8Item) -> Self {
        Self { item }
    }
}

impl Secs2Item for Secs2Uint8 {
    fn as_enum(self) -> Secs2Variant {
        Secs2Variant::UInt8(self)
    }

    fn length(&self) -> usize {
        self.item.len() * SECS2_UINT8_SIZE
    }
}

impl TryFrom<&[u8]> for Secs2Uint8 {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if (value.len() % SECS2_UINT8_SIZE) != 0 {
            return Err("input data size is invalid");
        }

        let result = value
            .chunks_exact(SECS2_UINT8_SIZE)
            .map(|chunk| {
                let mut val = 0u64;
                for i in 0..SECS2_UINT8_SIZE {
                    val <<= 8;
                    val |= chunk[i] as u64; 
                }
                val
            })
            .collect();

        Ok(Secs2Uint8::new(result))
    }
}
use crate::item::{Secs2Item, Secs2Variant};

type Secs2Uint2Item = Vec<u16>;
static SECS2_UINT2_SIZE: usize = 2;
#[derive(Debug)]
pub struct Secs2Uint2 {
    item: Secs2Uint2Item,
}

impl Secs2Uint2 {
    pub fn items(&self) -> &Secs2Uint2Item {
        &self.item
    }

    pub fn items_as_mut(&mut self) -> &mut Secs2Uint2Item {
        &mut self.item
    }

    pub fn new(item: Secs2Uint2Item) -> Self {
        Self { item }
    }
}

impl Secs2Item for Secs2Uint2 {
    fn as_enum(self) -> Secs2Variant {
        Secs2Variant::UInt2(self)
    }

    fn length(&self) -> usize {
        self.item.len() * SECS2_UINT2_SIZE
    }
}

impl TryFrom<&[u8]> for Secs2Uint2 {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if (value.len() % SECS2_UINT2_SIZE) != 0 {
            return Err("input data size is invalid");
        }

        let result = value
            .chunks_exact(SECS2_UINT2_SIZE)
            .map(|chunk| ((chunk[0] as u16) << 8) | (chunk[1] as u16))
            .collect();

        Ok(Secs2Uint2::new(result))
    }
}

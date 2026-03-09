use crate::{convert::secs2::serialize::Encode, item::{Secs2Item, Secs2Variant}};

type Secs2Uint4Item = Vec<u32>;
static SECS2_UINT4_SIZE: usize = 4;
#[derive(Debug)]
pub struct Secs2Uint4 {
    item: Secs2Uint4Item,
}

impl Secs2Uint4 {
    pub fn items(&self) -> &Secs2Uint4Item {
        &self.item
    }

    pub fn items_as_mut(&mut self) -> &mut Secs2Uint4Item {
        &mut self.item
    }

    pub fn new(item: Secs2Uint4Item) -> Self {
        Self { item }
    }
}

impl Secs2Item for Secs2Uint4 {
    fn as_enum(self) -> Secs2Variant {
        Secs2Variant::UInt4(self)
    }

    fn length(&self) -> usize {
        self.item.len() * SECS2_UINT4_SIZE
    }
}

impl Encode for Secs2Uint4 {
    fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), crate::error::Secs2Error> {
        for v in &self.item {
            w.write_all(&v.to_be_bytes())?;
        }

        Ok(())
    }
}

impl TryFrom<&[u8]> for Secs2Uint4 {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if (value.len() % SECS2_UINT4_SIZE) != 0 {
            return Err("input data size is invalid");
        }

        let result = value
            .chunks_exact(SECS2_UINT4_SIZE)
            .map(|chunk| {
                let mut val = 0u32;
                for i in 0..SECS2_UINT4_SIZE {
                    val <<= 8;
                    val |= chunk[i] as u32;
                }
                val
            })
            .collect();

        Ok(Secs2Uint4::new(result))
    }
}

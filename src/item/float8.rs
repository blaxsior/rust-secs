use crate::{convert::secs2::serialize::Encode, item::{Secs2Item, Secs2Variant}};

pub type Secs2Float8Item = Vec<f64>;
static SECS2_FLOAT8_SIZE: usize = 8;
#[derive(Debug)]
pub struct Secs2Float8 {
    item: Secs2Float8Item,
}

impl Secs2Float8 {
    pub fn items(&self) -> &Secs2Float8Item {
        &self.item
    }

    pub fn items_as_mut(&mut self) -> &mut Secs2Float8Item {
        &mut self.item
    }

    pub fn new(item: Secs2Float8Item) -> Self {
        Self { item }
    }
}

impl Secs2Item for Secs2Float8 {
    fn as_enum(self) -> Secs2Variant {
        Secs2Variant::Float8(self)
    }

    fn length(&self) -> usize {
        self.item.len() * SECS2_FLOAT8_SIZE
    }
}

impl Encode for Secs2Float8 {
    fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), crate::error::Secs2Error> {
        for v in &self.item {
            w.write_all(&v.to_be_bytes())?;
        }

        Ok(())
    }
}

impl TryFrom<&[u8]> for Secs2Float8 {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if (value.len() % SECS2_FLOAT8_SIZE) != 0 {
            return Err("input data size is invalid");
        }

        let result = value
            .chunks_exact(SECS2_FLOAT8_SIZE)
            .map(|chunk| {
                let arr: [u8; SECS2_FLOAT8_SIZE] =
                    chunk.try_into().expect("failed to convert bytes to F8");
                f64::from_be_bytes(arr)
            })
            .collect();

        Ok(Secs2Float8::new(result))
    }
}

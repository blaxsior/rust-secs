use crate::{convert::secs2::serialize::Encode, item::{Secs2Item, Secs2Variant}};

pub type Secs2Int1Item = Vec<i8>;
static SECS2_INT1_SIZE: usize = 1;
#[derive(Debug)]
pub struct Secs2Int1 {
    item: Secs2Int1Item,
}

impl Secs2Int1 {
    pub fn items(&self) -> &Secs2Int1Item {
        &self.item
    }

    pub fn items_as_mut(&mut self) -> &mut Secs2Int1Item {
        &mut self.item
    }

    pub fn new(item: Secs2Int1Item) -> Self {
        Self { item }
    }
}

impl Secs2Item for Secs2Int1 {
    fn as_enum(self) -> Secs2Variant {
        Secs2Variant::Int1(self)
    }

    fn length(&self) -> usize {
        self.item.len() // * SECS2_INT1_SIZE
    }
}

impl Encode for Secs2Int1 {
    fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), crate::error::Secs2Error> {
        for v in &self.item {
            w.write_all(&v.to_be_bytes())?;
        }

        Ok(())
    }
}

impl TryFrom<&[u8]> for Secs2Int1 {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if (value.len() % SECS2_INT1_SIZE) != 0 {
            return Err("input data size is invalid");
        }

        let result = value.into_iter().map(|chunk| *chunk as i8).collect();

        Ok(Secs2Int1::new(result))
    }
}

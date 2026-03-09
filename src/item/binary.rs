use crate::{convert::secs2::serialize::Encode, item::{Secs2Item, Secs2Variant}};

type Secs2BinaryItem = Vec<u8>;
#[derive(Debug)]
pub struct Secs2Binary {
    item: Secs2BinaryItem,
}

impl Secs2Binary {
    pub fn items(&self) -> &Secs2BinaryItem {
        &self.item
    }

    pub fn items_as_mut(&mut self) -> &mut Secs2BinaryItem {
        &mut self.item
    }

    pub fn new(item: Secs2BinaryItem) -> Self {
        Self { item }
    }
}

impl Secs2Item for Secs2Binary {
    fn as_enum(self) -> Secs2Variant {
        Secs2Variant::Binary(self)
    }

    fn length(&self) -> usize {
        self.item.len()
    }
}
impl Encode for Secs2Binary {
    fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), crate::error::Secs2Error> {
        w.write_all(self.items())?;
        Ok(())
    }
}


impl TryFrom<&[u8]> for Secs2Binary {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(Secs2Binary::new(value.to_vec()))
    }
}
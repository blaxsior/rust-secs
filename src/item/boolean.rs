use crate::{
    convert::secs2::serialize::Encode,
    item::{Secs2Item, Secs2Variant},
};

pub type Secs2BooleanItem = Vec<u8>;
pub type Secs2BooleanItem2 = Vec<bool>;

#[derive(Debug)]
pub struct Secs2Boolean {
    item: Secs2BooleanItem,
}

impl Secs2Boolean {
    pub fn items(&self) -> &Secs2BooleanItem {
        &self.item
    }

    pub fn items_as_mut(&mut self) -> &mut Secs2BooleanItem {
        &mut self.item
    }

    pub fn new(item: Secs2BooleanItem) -> Self {
        Self { item }
    }

    /// boolean 배열로부터 Secs2Boolean을 생성
    pub fn from(item: Secs2BooleanItem2) -> Self {
        Self::new(item.into_iter().map(|b| b as u8).collect::<Vec<u8>>())
    }

    pub fn is_true(&self, idx: usize) -> Option<bool> {
        self.item.get(idx).map(|&b| b > 0)
    }
}

impl Secs2Item for Secs2Boolean {
    fn as_enum(self) -> Secs2Variant {
        Secs2Variant::Boolean(self)
    }

    fn length(&self) -> usize {
        self.item.len()
    }
}

impl Encode for Secs2Boolean {
    fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), crate::error::Secs2Error> {
        w.write_all(self.items())?;
        Ok(())
    }
}

impl TryFrom<&[u8]> for Secs2Boolean {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(Secs2Boolean::new(value.to_vec()))
    }
}

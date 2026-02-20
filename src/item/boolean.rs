use crate::item::{Secs2Variant, Secs2Item};

type Secs2BooleanItem = Vec<u8>;
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
}

impl Secs2Item for Secs2Boolean {
    fn as_enum(self) -> Secs2Variant {
        Secs2Variant::Boolean(self)
    }

    fn length(&self) -> usize {
        self.item.len()
    }
}

impl TryFrom<&[u8]> for Secs2Boolean {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(Secs2Boolean::new(value.to_vec()))
    }
}

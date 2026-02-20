use crate::item::{Secs2Variant, Secs2Item};

type Secs2ASCIIItem = String;
pub struct Secs2ASCII {
    item: Secs2ASCIIItem,
}

impl Secs2ASCII {
    pub fn items(&self) -> &Secs2ASCIIItem {
        &self.item
    }

    pub fn items_as_mut(&mut self) -> &mut Secs2ASCIIItem {
        &mut self.item
    }

    pub fn new(item: Secs2ASCIIItem) -> Self {
        Self { item }
    }
}

impl Secs2Item for Secs2ASCII {
    fn as_enum(self) -> Secs2Variant {
        Secs2Variant::ASCII(self)
    }

    fn length(&self) -> usize {
        self.item.chars().count()
    }
}

impl TryFrom<&[u8]> for Secs2ASCII {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let result = String::from_utf8(value.to_vec()).expect("failed to parse buffer to ascii string");

        if result.is_ascii() {
            Ok(Self::new(result))
        } else {
            Err("result data is not ascii string")
        }
    }
}

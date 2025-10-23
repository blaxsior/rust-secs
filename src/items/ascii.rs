use crate::items::base::{Secs2ItemBody, Secs2Item};

type Secs2ASCIIItem = String;
pub struct Secs2ASCIIBody {
    item: Secs2ASCIIItem,
}

impl Secs2ASCIIBody {
    fn items(&self) -> &Secs2ASCIIItem {
        &self.item
    }

    fn items_as_mut(&mut self) -> &mut Secs2ASCIIItem {
        &mut self.item
    }

    fn new(item: Secs2ASCIIItem) -> Self {
        Self { item }
    }
}

impl Secs2ItemBody for Secs2ASCIIBody {
    fn as_enum(self) -> Secs2Item {
        Secs2Item::ASCII(self)
    }

    fn item_length(&self) -> usize {
        self.item.chars().count()
    }
}

impl ToString for Secs2ASCIIBody {
    fn to_string(&self) -> String {
        todo!()
    }
}

impl TryFrom<&[u8]> for Secs2ASCIIBody {
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



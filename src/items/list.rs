use crate::items::base::{Secs2ItemBody, Secs2FormatCode, Secs2Item};

type Secs2ListItem = Vec<Secs2Item>;

pub struct Secs2ListBody {
    item: Secs2ListItem,
}

impl Secs2ListBody {
    pub fn items(&self) -> &Secs2ListItem {
        &self.item
    }

    pub fn items_as_mut(&mut self) -> &mut Secs2ListItem {
        &mut self.item
    }

    pub fn new(item: Secs2ListItem) -> Self {
        Self { item }
    }
}

impl Secs2ItemBody for Secs2ListBody {
    fn as_enum(self) -> Secs2Item {
        return Secs2Item::List(self);
    }

    fn item_length(&self) -> usize {
        return self.item.len()
    }
}

impl ToString for Secs2ListBody {
    fn to_string(&self) -> String {
        todo!()
    }
}

// list item은 길이가 알려져 있지 않으므로 try-from으로 처리할 수 없음
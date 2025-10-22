use crate::items::base::{Secs2Item, Secs2ItemCode, Secs2ItemType};

type Secs2ListValue = Vec<Secs2ItemType>;

pub struct Secs2List {
    item: Secs2ListValue,
}

impl Secs2List {
    pub fn items(&self) -> &Secs2ListValue {
        &self.item
    }

    pub fn items_as_mut(&mut self) -> &mut Secs2ListValue {
        &mut self.item
    }

    pub fn new(item: Secs2ListValue) -> Self {
        Self { item }
    }
}

impl Secs2Item for Secs2List {
    fn as_enum(self) -> Secs2ItemType {
        return Secs2ItemType::List(self);
    }

    fn item_length(&self) -> usize {
        return self.item.len()
    }
    
    fn item_code() -> super::base::Secs2ItemCode {
        Secs2ItemCode::List
    }
}

impl ToString for Secs2List {
    fn to_string(&self) -> String {
        todo!()
    }
}

// list item은 길이가 알려져 있지 않으므로 try-from으로 처리할 수 없음
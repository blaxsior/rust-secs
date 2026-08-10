use crate::item::{Secs2Item, Secs2Variant};
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

pub type Secs2ListItem = Vec<Secs2Variant>;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Secs2List {
    item: Secs2ListItem,
}

impl Secs2List {
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

impl Secs2Item for Secs2List {
    fn as_enum(self) -> Secs2Variant {
        return Secs2Variant::List(self);
    }

    fn count(&self) -> usize {
        self.item.len()
    }

    fn length(&self) -> usize {
        return self.count();
    }
}

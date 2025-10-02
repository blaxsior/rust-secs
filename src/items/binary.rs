use crate::items::base::{Byte, Secs2Item};


pub struct Secs2Binary {
    items: Vec<Byte>
}

impl Secs2Item for Secs2Binary {
    type ItemType = Vec<Byte>;
    
    fn items(&self) -> &Self::ItemType {
        &self.items
    }
    
    fn items_as_mut(&mut self) -> &mut Self::ItemType {
        &mut self.items
    }
}
use crate::items::base::{Secs2Item, Secs2ItemCode, Secs2ItemType};

type Secs2Int1Value = Vec<i8>;
static SECS2_INT1_SIZE: usize = 1;
pub struct Secs2Int1 {
    item: Secs2Int1Value,
}

impl Secs2Int1 {
    fn items(&self) -> &Secs2Int1Value {
        &self.item
    }

    fn items_as_mut(&mut self) -> &mut Secs2Int1Value {
        &mut self.item
    }

    fn new(item: Secs2Int1Value) -> Self {
        Self { item }
    }
}

impl Secs2Item for Secs2Int1 {
    fn as_enum(self) -> Secs2ItemType {
        Secs2ItemType::Int1(self)
    }

    fn item_length(&self) -> usize {
        self.item.len()
    }
    
    fn item_code() -> super::base::Secs2ItemCode {
        Secs2ItemCode::Int1
    }
}

impl ToString for Secs2Int1 {
    fn to_string(&self) -> String {
        todo!()
    }
}

impl TryFrom<Vec<u8>> for Secs2Int1 {
    type Error = &'static str;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if (value.len() % SECS2_INT1_SIZE) != 0 {
            return Err("input data size is invalid");
        }

        let result = value.into_iter().map(|chunk| chunk as i8).collect();

        Ok(Secs2Int1::new(result))
    }
}

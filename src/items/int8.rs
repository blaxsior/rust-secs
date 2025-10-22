use crate::items::base::{Secs2Item, Secs2ItemCode, Secs2ItemType};

type Secs2Int8Value = Vec<i64>;
static SECS2_INT8_SIZE: usize = 8;

pub struct Secs2Int8 {
    item: Secs2Int8Value,
}

impl Secs2Int8 {
    fn items(&self) -> &Secs2Int8Value {
        &self.item
    }

    fn items_as_mut(&mut self) -> &mut Secs2Int8Value {
        &mut self.item
    }

     fn new(item: Secs2Int8Value) -> Self {
        Self { item }
    }
}


impl Secs2Item for Secs2Int8 {
    fn as_enum(self) -> super::base::Secs2ItemType {
        Secs2ItemType::Int8(self)
    }
    
    fn item_length(&self) -> usize {
        self.item.len() * SECS2_INT8_SIZE
    }

    fn item_code() -> super::base::Secs2ItemCode {
        Secs2ItemCode::Int8
    }
}

impl ToString for Secs2Int8 {
    fn to_string(&self) -> String {
        todo!()
    }
}

impl TryFrom<Vec<u8>> for Secs2Int8 {
    type Error = &'static str;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if (value.len() % SECS2_INT8_SIZE) != 0 {
            return Err("input data size is invalid");
        }

        let result = value
            .chunks_exact(SECS2_INT8_SIZE)
            .map(|chunk| {
                let arr: [u8; SECS2_INT8_SIZE] =
                    chunk.try_into().expect("failed to convert bytes to F8");
                i64::from_be_bytes(arr)
            })
            .collect();

        Ok(Secs2Int8::new(result))
    }
}

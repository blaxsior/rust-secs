use crate::items::base::{Secs2Item, Secs2ItemCode, Secs2ItemType};

type Secs2Float8Value = Vec<f64>;
static SECS2_FLOAT8_SIZE: usize = 8;
pub struct Secs2Float8 {
    item: Secs2Float8Value,
}

impl Secs2Float8 {
    fn items(&self) -> &Secs2Float8Value {
        &self.item
    }

    fn items_as_mut(&mut self) -> &mut Secs2Float8Value {
        &mut self.item
    }

    fn new(item: Secs2Float8Value) -> Self {
        Self { item }
    }
}

impl Secs2Item for Secs2Float8 {
    fn as_enum(self) -> Secs2ItemType {
        Secs2ItemType::Float8(self)
    }

    fn item_length(&self) -> usize {
        self.item.len() * SECS2_FLOAT8_SIZE
    }
    
    fn item_code() -> super::base::Secs2ItemCode {
        Secs2ItemCode::Float8
    }
}

impl ToString for Secs2Float8 {
    fn to_string(&self) -> String {
        todo!()
    }
}

impl TryFrom<Vec<u8>> for Secs2Float8 {
    type Error = &'static str;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if (value.len() % SECS2_FLOAT8_SIZE) != 0 {
            return Err("input data size is invalid");
        }

        let result = value.chunks_exact(SECS2_FLOAT8_SIZE).map(|chunk| {
            let arr: [u8; SECS2_FLOAT8_SIZE] = chunk.try_into().expect("failed to convert bytes to F8");
            f64::from_be_bytes(arr)
        }).collect();

        Ok(Secs2Float8::new(result))
    }
}

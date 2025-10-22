use crate::items::base::{Secs2Item, Secs2ItemCode, Secs2ItemType};

type Secs2Int4Value = Vec<i32>;
static SECS4_INT4_SIZE: usize = 4;
pub struct Secs2Int4 {
    item: Secs2Int4Value,
}

impl Secs2Int4 {
    fn items(&self) -> &Secs2Int4Value {
        &self.item
    }

    fn items_as_mut(&mut self) -> &mut Secs2Int4Value {
        &mut self.item
    }

    fn new(item: Secs2Int4Value) -> Self {
        Self { item }
    }
}

impl Secs2Item for Secs2Int4 {
    fn as_enum(self) -> Secs2ItemType {
        Secs2ItemType::Int4(self)
    }

    fn item_length(&self) -> usize {
        self.item.len() * SECS4_INT4_SIZE
    }
    
    fn item_code() -> super::base::Secs2ItemCode {
        Secs2ItemCode::Int4
    }
}

impl ToString for Secs2Int4 {
    fn to_string(&self) -> String {
        todo!()
    }
}

impl TryFrom<Vec<u8>> for Secs2Int4 {
    type Error = &'static str;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if (value.len() % SECS4_INT4_SIZE) != 0 {
            return Err("input data size is invalid");
        }

        let result = value
            .chunks_exact(SECS4_INT4_SIZE)
            .map(|chunk| {
                let arr: [u8; SECS4_INT4_SIZE] =
                    chunk.try_into().expect("failed to convert bytes to F8");
                i32::from_be_bytes(arr)
            })
            .collect();

        Ok(Secs2Int4::new(result))
    }
}

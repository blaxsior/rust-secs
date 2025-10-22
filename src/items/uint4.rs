use crate::items::base::{Secs2Item, Secs2ItemCode, Secs2ItemType};

type Secs2Uint4Value = Vec<u32>;
static SECS2_UINT4_SIZE: usize = 4;
pub struct Secs2Uint4 {
    item: Secs2Uint4Value,
}

impl Secs2Uint4 {
    fn items(&self) -> &Secs2Uint4Value {
        &self.item
    }

    fn items_as_mut(&mut self) -> &mut Secs2Uint4Value {
        &mut self.item
    }

    fn new(item: Secs2Uint4Value) -> Self {
        Self { item }
    }
}

impl Secs2Item for Secs2Uint4 {
    fn as_enum(self) -> Secs2ItemType {
        Secs2ItemType::UInt4(self)
    }

    fn item_length(&self) -> usize {
        self.item.len() * SECS2_UINT4_SIZE
    }
    
    fn item_code() -> super::base::Secs2ItemCode {
        Secs2ItemCode::UInt4
    }
}

impl ToString for Secs2Uint4 {
    fn to_string(&self) -> String {
        todo!()
    }
}

impl TryFrom<Vec<u8>> for Secs2Uint4 {
    type Error = &'static str;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if (value.len() % SECS2_UINT4_SIZE) != 0 {
            return Err("input data size is invalid");
        }

        let result = value
            .chunks_exact(SECS2_UINT4_SIZE)
            .map(|chunk| {
                let arr: [u8; SECS2_UINT4_SIZE] =
                    chunk.try_into().expect("failed to convert bytes to F8");
                u32::from_be_bytes(arr)
            })
            .collect();

        Ok(Secs2Uint4::new(result))
    }
}

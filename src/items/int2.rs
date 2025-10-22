use crate::items::base::{Secs2Item, Secs2ItemCode, Secs2ItemType};

type Secs2Int2Value = Vec<i16>;
static SECS2_INT2_SIZE: usize = 2;

pub struct Secs2Int2 {
    item: Secs2Int2Value,
}

impl Secs2Int2 {
    fn items(&self) -> &Secs2Int2Value {
        &self.item
    }

    fn items_as_mut(&mut self) -> &mut Secs2Int2Value {
        &mut self.item
    }

    fn new(item: Secs2Int2Value) -> Self {
        Self { item }
    }
}

impl Secs2Item for Secs2Int2 {
    fn as_enum(self) -> Secs2ItemType {
        Secs2ItemType::Int2(self)
    }

    fn item_length(&self) -> usize {
        self.item.len() * SECS2_INT2_SIZE
    }
    
    fn item_code() -> super::base::Secs2ItemCode {
        Secs2ItemCode::Int2
    }
}

impl ToString for Secs2Int2 {
    fn to_string(&self) -> String {
        todo!()
    }
}

impl TryFrom<Vec<u8>> for Secs2Int2 {
    type Error = &'static str;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if (value.len() % SECS2_INT2_SIZE) != 0 {
            return Err("input data size is invalid");
        }

        let result = value
            .chunks_exact(SECS2_INT2_SIZE)
            .map(|chunk| {
                let arr: [u8; SECS2_INT2_SIZE] =
                    chunk.try_into().expect("failed to convert bytes to F8");
                i16::from_be_bytes(arr)
            })
            .collect();

        Ok(Secs2Int2::new(result))
    }
}

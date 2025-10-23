use crate::items::base::{Secs2ItemBody, Secs2Item};

type Secs2Float4Item = Vec<f32>;
static SECS2_FLOAT4_SIZE: usize = 4;

pub struct Secs2Float4Body {
    item: Secs2Float4Item,
}

impl Secs2Float4Body {
    
    fn items(&self) -> &Secs2Float4Item {
        &self.item
    }

    fn items_as_mut(&mut self) -> &mut Secs2Float4Item {
        &mut self.item
    }

    fn new(item: Secs2Float4Item) -> Self {
        Self { item }
    }
}

impl Secs2ItemBody for Secs2Float4Body {
    fn as_enum(self) -> Secs2Item {
        Secs2Item::Float4(self)
    }
    
    fn item_length(&self) -> usize {
        self.item.len() * SECS2_FLOAT4_SIZE
    }
}

impl ToString for Secs2Float4Body {
    fn to_string(&self) -> String {
        todo!()
    }
}


impl TryFrom<&[u8]> for Secs2Float4Body {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if (value.len() % SECS2_FLOAT4_SIZE) != 0 {
            return Err("input data size is invalid");
        }

        let result = value.chunks_exact(SECS2_FLOAT4_SIZE).map(|chunk| {
            let arr: [u8; SECS2_FLOAT4_SIZE] = chunk.try_into().expect("failed to convert [u8; 4] to value");
            f32::from_be_bytes(arr)
        }).collect();

        Ok(Secs2Float4Body::new(result))
    }
}

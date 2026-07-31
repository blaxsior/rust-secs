use serde::{Serialize, de::DeserializeOwned};

use crate::model::file::codec::ModelCodec;

#[derive(Debug, Default, Clone, Copy)]
pub struct JsonCodec;

impl<T> ModelCodec<T> for JsonCodec
where
    T: Serialize + DeserializeOwned,
{
    type Error = serde_json::Error;

    fn decode(&self, bytes: &[u8]) -> Result<Vec<T>, Self::Error> {
        serde_json::from_slice(bytes)
    }

    fn encode(&self, items: &[T]) -> Result<Vec<u8>, Self::Error> {
        serde_json::to_vec_pretty(items)
    }
}

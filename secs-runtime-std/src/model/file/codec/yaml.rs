use std::collections::BTreeMap;

use serde::{Serialize, de::DeserializeOwned};

use crate::model::file::codec::ModelCodec;

#[derive(Debug, Default, Clone, Copy)]
pub struct YamlCodec;

impl<T> ModelCodec<T> for YamlCodec
where
    T: Serialize + DeserializeOwned,
{
    type Error = serde_yaml::Error;

    fn decode(&self, bytes: &[u8]) -> Result<BTreeMap<String, T>, Self::Error> {
        serde_yaml::from_slice(bytes)
    }

    fn encode(&self, items: &BTreeMap<String, T>) -> Result<Vec<u8>, Self::Error> {
        serde_yaml::to_string(items).map(String::into_bytes)
    }
}

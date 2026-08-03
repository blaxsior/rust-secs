use std::collections::BTreeMap;

pub mod json;
pub mod yaml;

pub use json::JsonCodec;
pub use yaml::YamlCodec;

pub trait ModelCodec<T> {
    type Error;

    fn decode(&self, bytes: &[u8]) -> Result<BTreeMap<String, T>, Self::Error>;

    fn encode(&self, items: &BTreeMap<String, T>) -> Result<Vec<u8>, Self::Error>;
}

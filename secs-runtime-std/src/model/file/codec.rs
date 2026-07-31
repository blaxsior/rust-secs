pub mod json;

pub use json::JsonCodec;

pub trait ModelCodec<T> {
    type Error;

    fn decode(&self, bytes: &[u8]) -> Result<Vec<T>, Self::Error>;

    fn encode(&self, items: &[T]) -> Result<Vec<u8>, Self::Error>;
}

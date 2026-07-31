use alloc::vec::Vec;

pub trait DataStore {
    type Error;

    fn load(&mut self, key: &str) -> Result<Option<Vec<u8>>, Self::Error>;

    fn save(&mut self, key: &str, bytes: &[u8]) -> Result<(), Self::Error>;

    fn remove(&mut self, key: &str) -> Result<(), Self::Error>;
}

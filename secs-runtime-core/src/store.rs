use alloc::vec::Vec;

pub trait DataStore<T> {
    type Error;

    fn load(&mut self) -> Result<(), Self::Error>;

    fn find(&mut self, key: &str) -> Result<Option<T>, Self::Error>;

    fn find_all(&mut self) -> Result<Vec<T>, Self::Error>;

    fn save(&mut self, key: &str, item: &T) -> Result<(), Self::Error>;

    fn save_all(&self) -> Result<(), Self::Error>;

    fn delete(&mut self, key: &str) -> Result<(), Self::Error>;
}

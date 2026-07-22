pub trait ByteDataSource {
    type Error;

    fn open(&mut self) -> Result<(), Self::Error>;

    fn close(&mut self) -> Result<(), Self::Error>;

    fn is_open(&self) -> bool;

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error>;

    fn write(&mut self, bytes: &[u8]) -> Result<(), Self::Error>;
}

#[derive(Debug)]
pub enum Secs2Error {
    Io(std::io::Error),
    InvalidFormatCode(u8),
    InvalidLength(usize),
    ParseError(String),
    Unimplemented
}

impl From<std::io::Error> for Secs2Error {
    fn from(e: std::io::Error) -> Self {
        Secs2Error::Io(e)
    }
}
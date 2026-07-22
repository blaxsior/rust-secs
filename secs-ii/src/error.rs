use alloc::string::String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Secs2Error {
    // 1. std::io::Error 대신 데이터가 부족할 때 발생하는 UnexpectedEof를 독자적 변수로 분리
    UnexpectedEof,
    
    // 2. 기존의 에러 포맷 유지
    InvalidFormatCode(u8),
    InvalidLength(usize),
    ParseError(String),
    Unimplemented,
}

// 💡 String이 에러 메세지로 바로 Secs2Error::ParseError로 변환될 수 있도록 From 구현
impl From<String> for Secs2Error {
    fn from(msg: String) -> Self {
        Secs2Error::ParseError(msg)
    }
}

// &str 리터럴도 바로 ? 연산자로 처리할 수 있게 From 추가해주면 유용합니다.
impl From<&str> for Secs2Error {
    fn from(msg: &str) -> Self {
        Secs2Error::ParseError(String::from(msg))
    }
}
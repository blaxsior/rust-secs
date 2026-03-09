use std::io::{Read};

/// cursor에 대한 read extension
pub trait ReadExt {
    // fn read_i8(&self) -> i8;
    // fn read_i16(&self) -> i16;
    // fn read_i32(&self) -> i32;
    // fn read_i64(&self) -> i64;

    // 1 byte u8 데이터를 읽어 반환한다.
    fn read_u8(&mut self) -> Result<u8, String>;
    // fn read_u16(&self) -> u16;
    // fn read_u32(&self) -> u32;
    // fn read_u64(&self) -> u64;

    // fn read_ascii(&self) -> String;
}

/// cursor에 대한 write extension
pub trait WriteExt {
    fn write_i8(&self);
    fn write_i16(&self);
    fn write_i32(&self);
    fn write_i64(&self);
    fn write_u8(&self);
    fn write_u16(&self);
    fn write_u32(&self);
    fn write_u64(&self);

    fn read_ascii(&self);
}

impl<R> ReadExt for R
where
    R: Read + ?Sized,
{
    fn read_u8(&mut self) -> Result<u8, String> {
        let mut buf = [0u8; 1];

        self.read_exact(&mut buf).map_err(|e| e.to_string())?;

        Ok(buf[0])
    }
}

use std::io::Cursor;

/// cursor에 대한 read extension
trait CursorReadExt {
    fn read_i8(&self) -> i8;
    fn read_i16(&self) -> i16;
    fn read_i32(&self) -> i32;
    fn read_i64(&self) -> i64;

    fn read_u8(&self) -> u8;
    fn read_u16(&self) -> u16;
    fn read_u32(&self) -> u32;
    fn read_u64(&self) -> u64;

    fn read_ascii(&self) -> String;
}

/// cursor에 대한 write extension
trait CursorWriteExt {
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

impl<T: AsRef<[u8]>> CursorReadExt for Cursor<T> {
    fn read_i8(&self) -> i8 {
        todo!()
    }

    fn read_i16(&self) -> i16 {
        todo!()
    }

    fn read_i32(&self) -> i32 {
        todo!()
    }

    fn read_i64(&self) -> i64 {
        todo!()
    }

    fn read_u8(&self) -> u8 {
        todo!()
    }

    fn read_u16(&self) -> u16 {
        todo!()
    }

    fn read_u32(&self) -> u32 {
        todo!()
    }

    fn read_u64(&self) -> u64 {
        todo!()
    }

    fn read_ascii(&self) -> String {
        todo!()
    }
}

use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream, ToSocketAddrs};
use std::time::Duration;

use secs_runtime_core::{ByteDataSource, ByteDataSourceError};

use super::{configure_stream, map_io_error};

pub struct TcpClientDataSource<A> {
    addr: A,
    stream: Option<TcpStream>,
    nonblocking: bool,
    read_timeout: Option<Duration>,
    write_timeout: Option<Duration>,
}

impl<A> TcpClientDataSource<A> {
    pub fn new(addr: A) -> Self {
        Self {
            addr,
            stream: None,
            nonblocking: true,
            read_timeout: None,
            write_timeout: None,
        }
    }

    pub fn with_nonblocking(mut self, nonblocking: bool) -> Self {
        self.nonblocking = nonblocking;
        self
    }

    pub fn with_read_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.read_timeout = timeout;
        self
    }

    pub fn with_write_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.write_timeout = timeout;
        self
    }
}

impl<A> ByteDataSource for TcpClientDataSource<A>
where
    A: ToSocketAddrs + Send,
{
    fn open(&mut self) -> Result<(), ByteDataSourceError> {
        if self.stream.is_some() {
            return Ok(());
        }

        let stream = TcpStream::connect(&self.addr).map_err(|_| ByteDataSourceError::OpenFailed)?;
        configure_stream(
            &stream,
            self.nonblocking,
            self.read_timeout,
            self.write_timeout,
        )?;

        self.stream = Some(stream);
        Ok(())
    }

    fn close(&mut self) -> Result<(), ByteDataSourceError> {
        let Some(stream) = self.stream.take() else {
            return Ok(());
        };

        stream
            .shutdown(Shutdown::Both)
            .map_err(|_| ByteDataSourceError::CloseFailed)
    }

    fn is_open(&self) -> bool {
        self.stream.is_some()
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ByteDataSourceError> {
        let Some(stream) = self.stream.as_mut() else {
            return Err(ByteDataSourceError::NotOpen);
        };

        let len = stream
            .read(buf)
            .map_err(|it| map_io_error(it, ByteDataSourceError::ReadFailed))?;
        if len == 0 {
            return Err(ByteDataSourceError::Disconnected);
        }

        Ok(len)
    }

    fn write(&mut self, bytes: &[u8]) -> Result<(), ByteDataSourceError> {
        let Some(stream) = self.stream.as_mut() else {
            return Err(ByteDataSourceError::NotOpen);
        };

        stream
            .write_all(bytes)
            .map_err(|it| map_io_error(it, ByteDataSourceError::WriteFailed))
    }
}

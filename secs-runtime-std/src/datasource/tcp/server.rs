use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream, ToSocketAddrs};
use std::time::Duration;

use secs_runtime_core::{ByteDataSource, ByteDataSourceError};

use super::{configure_stream, map_io_error};

pub struct TcpServerDataSource<A> {
    addr: A,
    listener: Option<TcpListener>,
    stream: Option<TcpStream>,
    stream_nonblocking: bool,
    read_timeout: Option<Duration>,
    write_timeout: Option<Duration>,
}

impl<A> TcpServerDataSource<A> {
    pub fn new(addr: A) -> Self {
        Self {
            addr,
            listener: None,
            stream: None,
            stream_nonblocking: true,
            read_timeout: None,
            write_timeout: None,
        }
    }

    pub fn with_stream_nonblocking(mut self, nonblocking: bool) -> Self {
        self.stream_nonblocking = nonblocking;
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

impl<A> ByteDataSource for TcpServerDataSource<A>
where
    A: ToSocketAddrs,
{
    fn open(&mut self) -> Result<(), ByteDataSourceError> {
        if self.stream.is_some() {
            return Ok(());
        }

        if self.listener.is_none() {
            log::info!("tcp server binding");
            let listener = TcpListener::bind(&self.addr).map_err(|error| {
                log::error!("failed to bind tcp server: {:?}", error);
                ByteDataSourceError::OpenFailed
            })?;
            log::info!("tcp server listening");
            self.listener = Some(listener);
        }

        let listener = self
            .listener
            .as_ref()
            .ok_or(ByteDataSourceError::OpenFailed)?;
        log::info!("tcp server waiting for peer");
        let (stream, _) = listener.accept().map_err(|error| {
            log::error!("failed to accept tcp peer: {:?}", error);
            ByteDataSourceError::OpenFailed
        })?;
        configure_stream(
            &stream,
            self.stream_nonblocking,
            self.read_timeout,
            self.write_timeout,
        )?;

        log::info!("peer connected!");
        self.stream = Some(stream);
        Ok(())
    }

    fn close(&mut self) -> Result<(), ByteDataSourceError> {
        let close_result = if let Some(stream) = self.stream.take() {
            stream
                .shutdown(Shutdown::Both)
                .map_err(|_| ByteDataSourceError::CloseFailed)
        } else {
            Ok(())
        };

        // self.listener = None;
        close_result
    }

    fn is_open(&self) -> bool {
        self.listener.is_some() || self.stream.is_some()
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

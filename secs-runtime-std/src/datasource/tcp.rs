mod client;
mod server;

use std::io::ErrorKind;
use std::net::TcpStream;
use std::time::Duration;

use secs_runtime_core::ByteDataSourceError;

pub use client::TcpClientDataSource;
pub use server::TcpServerDataSource;

pub type TcpDataSource<A> = TcpClientDataSource<A>;

fn configure_stream(
    stream: &TcpStream,
    nonblocking: bool,
    read_timeout: Option<Duration>,
    write_timeout: Option<Duration>,
) -> Result<(), ByteDataSourceError> {
    stream
        .set_nonblocking(nonblocking)
        .map_err(|_| ByteDataSourceError::OpenFailed)?;
    stream
        .set_read_timeout(read_timeout)
        .map_err(|_| ByteDataSourceError::OpenFailed)?;
    stream
        .set_write_timeout(write_timeout)
        .map_err(|_| ByteDataSourceError::OpenFailed)
}

fn map_io_error(error: std::io::Error, fallback: ByteDataSourceError) -> ByteDataSourceError {
    match error.kind() {
        ErrorKind::WouldBlock => ByteDataSourceError::WouldBlock,
        ErrorKind::TimedOut => ByteDataSourceError::TimedOut,
        ErrorKind::BrokenPipe
        | ErrorKind::ConnectionAborted
        | ErrorKind::ConnectionReset
        | ErrorKind::NotConnected
        | ErrorKind::UnexpectedEof => ByteDataSourceError::Disconnected,
        _ => fallback,
    }
}

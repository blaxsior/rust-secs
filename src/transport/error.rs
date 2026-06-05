use thiserror::Error;

use crate::transport::{SecsTimeoutUnit, secs1::{block::Secs1BlockHeader, config::DeviceId}};

///
/// SecsTransport 처리 시 예외
///
#[derive(Error, Debug, PartialEq, Eq)]
pub enum SecsTransportError {
    #[error("failed to connect")]
    ConnectionFailed(&'static str),

    #[error("failed to send message")]
    SendFailed,

    #[error("failed to receive message")]
    RecvFailed,

    #[error("connection closed")]
    ConnectionClosed,

    #[error("timeout: {0:?}")]
    Timeout(SecsTimeoutUnit),

    #[error("invalid block")]
    InvalidBlock,

    #[error("invalid block {0:?}")]
    UnexpectedBlock(Secs1BlockHeader),

    #[error("unknown device id: {0:?}")]
    UnknownDeviceId(DeviceId),

    #[error("invalid status")]
    InvalidState,
}


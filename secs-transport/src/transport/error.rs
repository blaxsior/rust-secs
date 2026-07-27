use alloc::string::String;
use secs_ii::{FunctionId, StreamId, error::Secs2Error};
use thiserror::Error;

use crate::transport::{
    DeviceId, SecsTimeoutUnit, TransactionKey, hsms::HsmsRejectReasonCode,
    secs1::block::Secs1BlockHeader,
};

///
/// SecsTransport 처리 시 예외
///
#[derive(Error, Debug, PartialEq, Eq)]
pub enum SecsTransportError {
    #[error("failed to connect")]
    ConnectionFailed(&'static str),

    #[error("failed to send message")]
    SendFailed(TransactionKey),

    #[error("failed to receive message")]
    RecvFailed,

    #[error("connection closed")]
    ConnectionClosed,

    #[error("timeout: {0:?}")]
    Timeout(SecsTimeoutUnit),

    #[error("invalid header")]
    InvalidHeader,

    #[error("invalid HSMS header: {0:?}")]
    InvalidHsmsHeader(HsmsHeaderError),

    #[error("invalid block length. expected [10..254], found {0}")]
    InvalidBlockLength(usize),

    #[error("message convert failed: {0:?}")]
    MessageConvertFailed(SecsMessageConvertError),

    #[error("invalid block")]
    BlockError,

    #[error("unexpected block {0:?}")]
    UnexpectedBlock(Secs1BlockHeader),

    #[error("unknown device id: {0:?}")]
    UnknownDeviceId(DeviceId),

    #[error("invalid state found")]
    InvalidState,

    #[error("no match SxFy {0:?} {1:?}")]
    NoMatchReplyTransaction(StreamId, FunctionId),

    #[error("no such transaction {0:?}")]
    NoSuchTransaction(TransactionKey),
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum HsmsHeaderError {
    #[error("unsupported p-type: {0}")]
    UnsupportedPType(u8),

    #[error("unsupported s-type: {0}")]
    UnsupportedSType(u8),
}

impl HsmsHeaderError {
    pub fn reject_reason(&self) -> Option<HsmsRejectReasonCode> {
        match self {
            Self::UnsupportedPType(_) => Some(HsmsRejectReasonCode::NotSupportedPType),
            Self::UnsupportedSType(_) => Some(HsmsRejectReasonCode::NotSupportedSType),
        }
    }
}

///
/// secs 메시지 <-> block 변환 시 예외
///
#[derive(Error, Debug, PartialEq, Eq)]
pub enum SecsMessageConvertError {
    #[error("block no is not sequential: {0:?}")]
    SequenceGap(u16),

    #[error("e-bit is missing")]
    MissingEbit,

    #[error("no blocks to convert. blocks is empty")]
    EmptyBlocks,

    #[error("decoding error {0:?}")]
    DecodeFailed(String),

    #[error("encoding error {0:?}")]
    EncodeFailed(Secs2Error),
}

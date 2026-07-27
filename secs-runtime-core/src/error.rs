use secs_common::TransactionKey;

use crate::io::ByteDataSourceError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachineError {
    InvalidState,
    InvalidMessage,
    InvalidTimeout,
    EncodeFailed,
    DecodeFailed,
    SendFailed(TransactionKey),
    ReceiveFailed(TransactionKey),
    TransportSpecific(u16),
}

pub enum RuntimeError<M, T> {
    DataSource(ByteDataSourceError),
    Machine(M),
    Timer(T),
}

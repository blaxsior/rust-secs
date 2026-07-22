use secs_common::TransactionKey;

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

pub enum RuntimeError<D, M, T> {
    DataSource(D),
    Machine(M),
    Timer(T),
}

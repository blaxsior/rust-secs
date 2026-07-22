pub mod error;
pub mod hsms;
pub mod secs1;

pub use secs_common::{
    ConnectionRole, DeviceId, SessionId, SystemByte, TransactionKey, TransactionOwner,
    TransferContext,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecsTimeoutUnit {
    T1,
    T2,
    T3(TransactionKey),
    T4(TransactionKey),
    T5,
    T6,
    T7,
    T8,
}

impl SecsTimeoutUnit {
    pub fn to_transaction_key(&self) -> TransactionKey {
        match *self {
            SecsTimeoutUnit::T3(key) => key,
            SecsTimeoutUnit::T4(key) => key,
            _ => panic!("unexpected condition {:?}", self),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rbit(bool);

impl Rbit {
    pub const FORWARD: Self = Self(false);
    pub const REVERSE: Self = Self(true);

    pub const fn complement(self) -> Self {
        Self(!self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Wbit(bool);

impl Wbit {
    pub fn need_reply(&self) -> bool {
        self.0
    }
}

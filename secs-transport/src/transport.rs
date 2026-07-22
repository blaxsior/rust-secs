pub mod error;
pub mod hsms;
pub mod secs1;

pub use secs_common::{
    ConnectionRole, DeviceId, SecsTimeoutUnit, SessionId, SystemByte, TimeoutId, TimeoutTicket,
    TransactionKey, TransactionOwner, TransferContext,
};

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

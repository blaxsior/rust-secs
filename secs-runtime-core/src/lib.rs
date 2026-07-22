#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub mod error;
pub mod io;
pub mod machine;
pub mod message;
pub mod timer;

pub use error::{MachineError, RuntimeError};
pub use io::ByteDataSource;
pub use machine::{MachineEvent, MachineSignal, MessageMachine};
pub use message::{RuntimeMessage, SystemByteSource};
pub use timer::{RuntimeTimeout, RuntimeTimer, TimeoutTicket};

pub use secs_common::{
    ConnectionRole, DeviceId, SessionId, SystemByte, TransactionKey, TransactionOwner,
    TransferContext,
};

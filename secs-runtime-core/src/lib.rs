#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub mod client;
pub mod error;
pub mod io;
pub mod machine;
pub mod message;
pub mod timer;

pub use client::{TransportClient, TransportClientEvent};
pub use error::{MachineError, RuntimeError};
pub use io::ByteDataSource;
pub use machine::{MachineEvent, MachineSignal, MessageMachine};
pub use message::{RuntimeMessage, SystemByteSource};
pub use timer::{RuntimeTimer, TimeoutTicket, Timer};

pub use secs_common::{
    ConnectionRole, DeviceId, SecsTimeoutUnit, SessionId, SystemByte, TimeoutId, TransactionKey,
    TransactionOwner, TransferContext,
};

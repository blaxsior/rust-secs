#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub mod error;
pub mod io;
pub mod machine;
pub mod message;
pub mod promise;
pub mod store;
pub mod timer;

pub use error::{MachineError, RuntimeError};
pub use io::{ByteDataSource, ByteDataSourceError};
pub use machine::{MachineEvent, MachineSignal, MessageTransport};
pub use message::RuntimeMessage;
pub use promise::{
    PromiseFuture, PromiseResolver, TaskFuture, TaskQueue, TaskRunner, TaskSpawnError, promise,
};
pub use store::DataStore;
pub use timer::{SecsTimer, TimeoutTicket};

pub use secs_common::{
    ConnectionRole, DeviceId, SecsTimeoutUnit, SessionId, SystemByte, SystemByteSource, TimeoutId,
    TransactionKey, TransactionOwner, TransferContext,
};

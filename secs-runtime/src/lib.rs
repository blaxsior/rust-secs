#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub mod core;
pub mod machine;
pub mod message;
pub mod secs2;

pub use crate::core::{
    ByteDataSource, ConnectionRole, DeviceId, MachineError, MachineEvent, MachineSignal,
    MessageMachine, RuntimeError, RuntimeMessage, RuntimeTimer, SessionId, SystemByte,
    SystemByteSource, TimeoutId, TimeoutTicket, Timer, TransactionKey, TransactionOwner,
    TransferContext,
};
pub use crate::machine::{SecsMachine, SecsMachineError};
pub use crate::message::{MessageRuntime, MessageRuntimeEvent, MessageRuntimeTick};
pub use crate::secs2::{
    CallToken, ExchangeTracker, IgnoreUnknownPrimary, PendingCall, Secs2DefaultHandler,
    Secs2MessageHandler, Secs2MessageLayer, Secs2PrimaryHandler, Secs2RequestContext, Secs2Route,
    Secs2Router, Secs2Runtime, Secs2RuntimeError, Secs2RuntimeEvent,
};

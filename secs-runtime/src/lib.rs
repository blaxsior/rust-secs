#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub mod core;
pub mod machine;
pub mod message;
pub mod secs2;

pub use crate::core::{
    ByteDataSource, CorrelationId, CorrelationIdSource, RuntimeMessage, RuntimeTimer,
};
pub use crate::message::{
    MessageMachine, MessageRuntime, MessageRuntimeEvent, MessageRuntimeTick, RuntimeError,
};
pub use crate::machine::{SecsMachine, SecsMachineError};
pub use crate::secs2::{
    CallToken, ExchangeTracker, IgnoreUnknownPrimary, PendingCall, Secs2DefaultHandler,
    Secs2MessageHandler, Secs2MessageLayer, Secs2PrimaryHandler, Secs2RequestContext, Secs2Route,
    Secs2Router, Secs2Runtime, Secs2RuntimeError, Secs2RuntimeEvent,
};

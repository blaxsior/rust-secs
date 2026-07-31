#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub mod error;
pub mod runtime;
pub mod shared;
pub mod timer;

pub use error::{CallError, HandlerError, SecsRuntimeError};
pub use runtime::SecsRuntime;
pub use shared::{InboundFuture, RecvFuture, SecsHandle};
pub use timer::TimeoutConfig;

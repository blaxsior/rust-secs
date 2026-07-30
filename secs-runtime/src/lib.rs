#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub mod error;
pub mod runtime;
pub mod scenario;
pub mod service;
mod shared;
pub mod timer;

pub use error::{CallError, HandlerError, SecsRuntimeError};
pub use runtime::{DefaultScenarioTaskRunner, ScenarioTaskOutput, SecsRuntime, SecsRuntimeRoute};
pub use scenario::{BoxFuture, ScenarioContext, SecsScenario};
pub use service::{SecsService, ServiceContext};
pub use timer::TimeoutConfig;

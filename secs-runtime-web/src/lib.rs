//! Web/WASM adapters for `secs-runtime`.
//!
//! This crate is intentionally thin for now. Runtime-independent contracts stay in
//! `secs-runtime-core`, while browser-specific task/timer/transport adapters can be
//! added here.

pub mod bind;
pub mod datasource;
pub mod logger;
pub mod runtime;
pub mod timer;

pub use datasource::{WebDataSource, WebDataSourceHandle};
pub use logger::{WebLogger, init_logger, init_logger_with_callback, set_log_callback};
pub use runtime::{WebRuntime, WebRuntimeError, hsms_timeout_config};
pub use timer::WebSecsTimer;
pub use wasm_bindgen;

#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub mod context;
pub mod error;
pub mod handler;

pub use context::SecsContext;
pub use error::SecsHandlerError;
pub use handler::{
    BoxSecsActionFuture, BoxSecsRouteFuture, SecsAction, SecsHandler, SecsRoute,
    SecsMatcher, SecsRouteHandler,
};

#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub mod error;
pub mod domain;
pub mod dictionary;

pub use error::SecsModelError;
pub use domain::{EventEntry, ReportEntry, SecsKey, ValueBinding, ValueEntry};
pub use dictionary::SecsDictionary;

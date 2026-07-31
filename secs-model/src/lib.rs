#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub mod error;
pub mod domain;
pub mod store;

pub use error::SecsModelError;
pub use domain::{
    EventId, EventSpec, EventDictionary, ReportId, ReportSpec, ReportDictionary, ValueId, ValueSpec,
    ValueData, ValueDictionary,
};
pub use store::{
    EventRepository, NoopEventRepository, NoopReportRepository, NoopValueDataRepository,
    NoopValueSpecRepository, ReportRepository, StoreError, ValueDataRepository,
    ValueSpecRepository,
};

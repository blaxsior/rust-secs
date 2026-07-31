pub(crate) mod default_policy {
    pub(crate) fn persistent() -> bool {
        true
    }

    pub(crate) fn readonly() -> bool {
        true
    }

}

pub mod event;
pub mod report;
pub mod value;

pub use event::{EventId, EventSpec, EventDictionary};
pub use report::{ReportId, ReportSpec, ReportDictionary};
pub use value::{ValueData, ValueId, ValueSpec, ValueDictionary};

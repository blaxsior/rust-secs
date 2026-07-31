use secs_ii::item::Secs2FormatCode;

use crate::{EventId, ReportId, StoreError, ValueId};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SecsModelError {
    #[error("unknown value key: {0:?}")]
    UnknownValue(ValueId),

    #[error("unknown report key: {0:?}")]
    UnknownReport(ReportId),

    #[error("unknown event key: {0:?}")]
    UnknownEvent(EventId),

    #[error("value is read only: {0:?}")]
    ReadOnlyValue(ValueId),

    #[error("report is read only: {0:?}")]
    ReadOnlyReport(ReportId),

    #[error("event is read only: {0:?}")]
    ReadOnlyEvent(EventId),

    #[error("invalid value format for {id:?}: expected {expected:?}, actual {actual:?}")]
    InvalidValueFormat {
        id: ValueId,
        expected: Secs2FormatCode,
        actual: Secs2FormatCode,
    },

    #[error("failed to encode value: {0:?}")]
    EncodeValue(ValueId),

    #[error("failed to decode value: {0:?}")]
    DecodeValue(ValueId),

    #[error("store error: {0}")]
    Store(#[from] StoreError),
}

use crate::SecsKey;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SecsModelError {
    #[error("unknown value key: {0:?}")]
    UnknownValue(SecsKey),

    #[error("unknown report key: {0:?}")]
    UnknownReport(SecsKey),

    #[error("unknown event key: {0:?}")]
    UnknownEvent(SecsKey),

    #[error("value is read only: {0:?}")]
    ReadOnlyValue(SecsKey),
}

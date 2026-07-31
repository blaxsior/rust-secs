use secs_runtime::CallError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SecsHandlerError {
    #[error("SECS call failed: {0:?}")]
    Call(CallError),

    #[error("SECS route is not registered")]
    RouteNotFound,

    #[error("SECS action is not registered")]
    ActionNotFound,

    #[error("SECS handler failed")]
    Failed,
}

impl From<CallError> for SecsHandlerError {
    fn from(value: CallError) -> Self {
        Self::Call(value)
    }
}

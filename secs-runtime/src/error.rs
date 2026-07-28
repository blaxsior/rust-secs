use secs_runtime_core::MachineError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallError {
    Timeout,
    Transport(MachineError),
    UnknownToken,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandlerError {
    Failed,
    Call(CallError),
    Transport(MachineError),
}

impl From<CallError> for HandlerError {
    fn from(value: CallError) -> Self {
        Self::Call(value)
    }
}

impl From<MachineError> for HandlerError {
    fn from(value: MachineError) -> Self {
        Self::Transport(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecsRuntimeError<TimerError> {
    Transport(MachineError),
    Timer(TimerError),
    Handler(HandlerError),
}

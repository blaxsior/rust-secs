use secs_common::TransactionKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RuntimeTimeout {
    T1,
    T2,
    T3(TransactionKey),
    T4(TransactionKey),
    T5,
    T6,
    T7,
    T8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeoutTicket {
    pub timeout: RuntimeTimeout,
    pub generation: u32,
}

pub trait RuntimeTimer {
    type Error;

    fn start(&mut self, timeout: RuntimeTimeout) -> Result<TimeoutTicket, Self::Error>;

    fn cancel(&mut self, timeout: RuntimeTimeout) -> Result<(), Self::Error>;

    fn poll_timeout(&mut self) -> Result<Option<TimeoutTicket>, Self::Error>;
}

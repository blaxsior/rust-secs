pub use secs_common::TimeoutTicket;

pub trait Timer {
    type Error;
    type Duration;
    type Handle;

    fn start_after(&mut self, duration: Self::Duration) -> Result<Self::Handle, Self::Error>;

    fn cancel(&mut self, handle: Self::Handle) -> Result<(), Self::Error>;
}

pub trait RuntimeTimer: Timer {
    fn start_secs_timeout(&mut self, ticket: TimeoutTicket) -> Result<Self::Handle, Self::Error>;

    fn cancel_secs_timeout(&mut self, handle: Self::Handle) -> Result<(), Self::Error>;

    fn poll_secs_timeout(&mut self) -> Result<Option<TimeoutTicket>, Self::Error>;
}

pub use secs_common::TimeoutTicket;

pub trait SecsTimer {
    type Error;
    type Duration;
    type Handle;

    fn start_timeout(
        &mut self,
        ticket: TimeoutTicket,
        duration: Self::Duration,
    ) -> Result<Self::Handle, Self::Error>;

    fn poll_timeout(&mut self) -> Result<Option<TimeoutTicket>, Self::Error>;
}

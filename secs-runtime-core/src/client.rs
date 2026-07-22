/// Common lifecycle events exposed by protocol-specific clients.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportClientEvent {
    LinkOpened,
    LinkClosed,
}

/// Common client abstraction for concrete transport protocols.
///
/// This trait is intentionally below the SECS-II request/reply layer. A
/// `Secs1Client` should use its own SECS-I message type, and an `HsmsClient`
/// should use its own HSMS message type. Higher layers may wrap this trait to
/// provide stream/function based APIs.
pub trait TransportClient {
    type Error;
    type Message;

    /// Start the protocol-specific connection/session process.
    fn start(&mut self) -> Result<(), Self::Error>;

    /// Stop the protocol-specific connection/session process.
    fn stop(&mut self) -> Result<(), Self::Error>;

    /// Drive data source, timer, and internal protocol progress once.
    fn tick(&mut self) -> Result<(), Self::Error>;

    /// Returns whether protocol data messages can currently be sent.
    fn is_ready(&self) -> bool;

    /// Queue one protocol-specific outbound message.
    fn send(&mut self, message: Self::Message) -> Result<(), Self::Error>;

    /// Poll one protocol-specific inbound message.
    fn poll_recv(&mut self) -> Option<Self::Message>;
}

use secs_ii::{FunctionId, Secs2Message, StreamId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CorrelationId(pub u32);

pub trait CorrelationIdSource {
    fn next_correlation_id(&mut self) -> CorrelationId;
}

pub trait ByteDataSource {
    type Error;

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error>;

    fn write(&mut self, bytes: &[u8]) -> Result<(), Self::Error>;
}

pub trait RuntimeTimer {
    type Error;
    type Timeout;
    type Ticket;

    fn start(&mut self, timeout: Self::Timeout) -> Result<Self::Ticket, Self::Error>;

    fn cancel(&mut self, timeout: Self::Timeout) -> Result<(), Self::Error>;

    fn poll_timeout(&mut self) -> Result<Option<Self::Ticket>, Self::Error>;
}

pub struct RuntimeMessage {
    pub correlation_id: CorrelationId,
    pub payload: Secs2Message,
}

impl RuntimeMessage {
    pub fn new(correlation_id: CorrelationId, payload: Secs2Message) -> Self {
        Self {
            correlation_id,
            payload,
        }
    }

    pub fn stream(&self) -> StreamId {
        self.payload.stream
    }

    pub fn function(&self) -> FunctionId {
        self.payload.function
    }

    pub fn is_primary(&self) -> bool {
        self.payload.function.is_primary()
    }

    pub fn is_secondary(&self) -> bool {
        self.payload.function.is_secondary()
    }

    pub fn need_reply(&self) -> bool {
        self.payload.need_reply
    }

    pub fn into_payload(self) -> Secs2Message {
        self.payload
    }
}

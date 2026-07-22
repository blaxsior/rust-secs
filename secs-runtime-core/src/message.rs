use secs_common::{SystemByte, TransactionKey, TransactionOwner};
use secs_ii::{FunctionId, Secs2Message, StreamId};

pub trait SystemByteSource {
    fn next_system_byte(&mut self) -> SystemByte;
}

pub struct RuntimeMessage {
    pub transaction_key: TransactionKey,
    pub payload: Secs2Message,
}

impl RuntimeMessage {
    pub fn new(transaction_key: TransactionKey, payload: Secs2Message) -> Self {
        Self {
            transaction_key,
            payload,
        }
    }

    pub fn new_local(system_byte: SystemByte, payload: Secs2Message) -> Self {
        Self::new(
            TransactionKey::new(TransactionOwner::Local, system_byte),
            payload,
        )
    }

    pub fn new_remote(system_byte: SystemByte, payload: Secs2Message) -> Self {
        Self::new(
            TransactionKey::new(TransactionOwner::Remote, system_byte),
            payload,
        )
    }

    pub fn system_byte(&self) -> SystemByte {
        self.transaction_key.system_byte
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

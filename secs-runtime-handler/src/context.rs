use secs_ii::Secs2Message;
use secs_runtime::{InboundFuture, RecvFuture, SecsHandle};
use secs_runtime_core::{RuntimeMessage, TransactionKey};

#[derive(Clone)]
pub struct SecsContext {
    handle: SecsHandle,
}

impl SecsContext {
    pub fn new(handle: SecsHandle) -> Self {
        Self { handle }
    }

    pub fn handle(&self) -> &SecsHandle {
        &self.handle
    }

    pub fn send(&self, message: Secs2Message) -> Option<TransactionKey> {
        self.handle.send(message)
    }

    pub fn send_with_key(&self, key: TransactionKey, message: Secs2Message) {
        self.handle.send_with_key(key, message);
    }

    pub fn request(&self, message: Secs2Message) -> RecvFuture {
        self.handle.request(message)
    }

    pub fn recv(&self) -> InboundFuture {
        self.handle.recv()
    }

    pub fn recv_call(&self, key: TransactionKey) -> RecvFuture {
        self.handle.recv_call(key)
    }

    pub fn transaction_key(&self, message: &RuntimeMessage) -> TransactionKey {
        message.transaction_key
    }
}

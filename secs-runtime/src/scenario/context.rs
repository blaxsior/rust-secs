use secs_ii::Secs2Message;
use secs_runtime_core::TransactionKey;

use crate::shared::{RecvFuture, RuntimeHandle};

#[derive(Clone)]
pub struct ScenarioContext {
    handle: RuntimeHandle,
}

impl ScenarioContext {
    pub(crate) fn new(handle: RuntimeHandle) -> Self {
        Self { handle }
    }

    pub fn send(&self, message: Secs2Message) -> Option<TransactionKey> {
        self.handle.send(message)
    }

    pub fn recv(&self, key: TransactionKey) -> RecvFuture {
        self.handle.recv_call(key)
    }
}

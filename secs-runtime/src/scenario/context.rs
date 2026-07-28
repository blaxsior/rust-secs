use secs_ii::Secs2Message;
use secs_runtime_core::TransactionKey;

use crate::{
    error::HandlerError,
    shared::{RecvFuture, RuntimeHandle},
};

#[derive(Clone)]
pub struct ScenarioContext {
    handle: RuntimeHandle,
}

impl ScenarioContext {
    pub(crate) fn new(handle: RuntimeHandle) -> Self {
        Self { handle }
    }

    pub fn send(&self, message: Secs2Message) -> Option<TransactionKey> {
        self.handle.send_scenario_message(message)
    }

    pub fn recv(&self, key: TransactionKey) -> RecvFuture {
        self.handle.recv_call(key)
    }

    pub fn call(&self, primary: Secs2Message) -> Result<RecvFuture, HandlerError> {
        let token = self.send(primary).ok_or(HandlerError::Failed)?;
        Ok(self.recv(token))
    }
}

use secs_ii::Secs2Message;
use secs_runtime_core::{RuntimeMessage, TransactionKey, TransactionOwner};

use crate::{error::HandlerError, shared::RuntimeHandle};

pub struct ServiceContext {
    handle: RuntimeHandle,
    message: Option<Secs2Message>,
    message_key: TransactionKey,
}

impl ServiceContext {
    pub(crate) fn new(handle: RuntimeHandle, message: RuntimeMessage) -> Self {
        Self {
            handle,
            message: Some(message.payload),
            message_key: message.transaction_key,
        }
    }

    pub fn recv(&mut self) -> Option<Secs2Message> {
        self.message.take()
    }

    pub fn send(&mut self, message: Secs2Message) -> Result<(), HandlerError> {
        self.handle.send_only(RuntimeMessage::new(
            TransactionKey::new(TransactionOwner::Local, self.message_key.system_byte),
            message,
        ));
        Ok(())
    }
}

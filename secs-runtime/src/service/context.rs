use secs_ii::Secs2Message;
use secs_runtime_core::{RuntimeMessage, TransactionKey};

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

    /// 상대방이 보낸 메시지를 받아 사용
    pub fn recv(&mut self) -> Option<Secs2Message> {
        self.message.take()
    }

    /// 상대방 메시지에 응답
    pub fn reply(&mut self, message: Secs2Message) -> Result<(), HandlerError> {
        self.handle.reply(self.message_key, message);
        Ok(())
    }
}

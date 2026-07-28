use alloc::{
    collections::{BTreeMap, VecDeque},
    rc::Rc,
};
use core::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use secs_ii::Secs2Message;
use secs_runtime_core::{RuntimeMessage, SystemByteSource, TransactionKey, TransactionOwner};

use crate::error::CallError;

pub(crate) enum RuntimeCommand {
    Send {
        message: RuntimeMessage,
        call: Option<TransactionKey>,
    },
}

pub(crate) struct PendingCall {
    pub result: Option<Result<Secs2Message, CallError>>,
}

pub(crate) struct RuntimeShared {
    system_bytes: SystemByteSource,
    pub commands: VecDeque<RuntimeCommand>,
    pub pending_calls: BTreeMap<TransactionKey, PendingCall>,
}

impl RuntimeShared {
    pub(crate) fn new(system_bytes: SystemByteSource) -> Self {
        Self {
            system_bytes,
            commands: VecDeque::new(),
            pending_calls: BTreeMap::new(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct RuntimeHandle {
    shared: Rc<RefCell<RuntimeShared>>,
}

impl RuntimeHandle {
    pub(crate) fn new(shared: Rc<RefCell<RuntimeShared>>) -> Self {
        Self { shared }
    }

    pub(crate) fn send_scenario_message(&self, message: Secs2Message) -> Option<TransactionKey> {
        let mut shared = self.shared.borrow_mut();
        let system_byte = shared.system_bytes.next_system_byte();
        let key = TransactionKey::new(TransactionOwner::Local, system_byte);
        let should_wait_reply = message.function.is_primary() && message.need_reply;
        let call_key = should_wait_reply.then_some(key);
        let message = RuntimeMessage::new(key, message);

        if let Some(call_key) = call_key {
            shared
                .pending_calls
                .insert(call_key, PendingCall { result: None });
        }
        shared.commands.push_back(RuntimeCommand::Send {
            message,
            call: call_key,
        });

        call_key
    }

    pub(crate) fn send_only(&self, message: RuntimeMessage) {
        let mut shared = self.shared.borrow_mut();
        shared.commands.push_back(RuntimeCommand::Send {
            message,
            call: None,
        });
    }

    pub(crate) fn recv_call(&self, key: TransactionKey) -> RecvFuture {
        RecvFuture {
            shared: self.shared.clone(),
            key,
        }
    }
}

pub struct RecvFuture {
    shared: Rc<RefCell<RuntimeShared>>,
    key: TransactionKey,
}

impl Future for RecvFuture {
    type Output = Result<Secs2Message, CallError>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared = self.shared.borrow_mut();
        let Some(pending) = shared.pending_calls.get_mut(&self.key) else {
            return Poll::Ready(Err(CallError::UnknownToken));
        };

        let Some(result) = pending.result.take() else {
            return Poll::Pending;
        };

        shared.pending_calls.remove(&self.key);

        Poll::Ready(result)
    }
}

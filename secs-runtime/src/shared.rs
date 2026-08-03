use alloc::{
    collections::{BTreeMap, VecDeque},
};
use std::sync::{Arc, Mutex, MutexGuard};

use secs_ii::Secs2Message;
use secs_runtime_core::{
    PromiseFuture, PromiseResolver, RuntimeMessage, SystemByteSource, TransactionKey,
    TransactionOwner, promise,
};

use crate::error::CallError;

type SharedRuntime = Arc<Mutex<RuntimeShared>>;

pub(crate) enum RuntimeCommand {
    Send {
        message: RuntimeMessage,
        call: Option<TransactionKey>,
    },
}

pub(crate) struct PendingCall {
    pub resolver: Option<PromiseResolver<Secs2Message, CallError>>,
}

pub(crate) struct RuntimeShared {
    system_bytes: SystemByteSource,
    pub commands: VecDeque<RuntimeCommand>,
    pub pending_calls: BTreeMap<TransactionKey, PendingCall>,
    pub incomming_msgs: VecDeque<RuntimeMessage>,
    recv_waiters: VecDeque<PromiseResolver<RuntimeMessage, CallError>>,
}

impl RuntimeShared {
    pub(crate) fn new(system_bytes: SystemByteSource) -> Self {
        Self {
            system_bytes,
            commands: VecDeque::new(),
            pending_calls: BTreeMap::new(),
            incomming_msgs: VecDeque::new(),
            recv_waiters: VecDeque::new(),
        }
    }

    pub(crate) fn send(&mut self, message: Secs2Message) -> Option<TransactionKey> {
        let system_byte = self.system_bytes.next_system_byte();
        let key = TransactionKey::new(TransactionOwner::Local, system_byte);
        self.send_with_key(key, message)
    }

    pub(crate) fn request(&mut self, message: Secs2Message) -> RecvFuture {
        let (resolver, future) = promise();
        let system_byte = self.system_bytes.next_system_byte();
        let key = TransactionKey::new(TransactionOwner::Local, system_byte);
        let should_wait_reply = message.function.is_primary() && message.need_reply;
        let message = RuntimeMessage::new(key, message);

        if should_wait_reply {
            self.pending_calls.insert(
                key,
                PendingCall {
                    resolver: Some(resolver),
                },
            );
        } else {
            resolver.reject(CallError::UnknownToken);
        }

        self.commands.push_back(RuntimeCommand::Send {
            message,
            call: should_wait_reply.then_some(key),
        });

        future
    }

    pub(crate) fn send_with_key(
        &mut self,
        key: TransactionKey,
        message: Secs2Message,
    ) -> Option<TransactionKey> {
        let should_wait_reply = message.function.is_primary() && message.need_reply;
        let call_key = should_wait_reply.then_some(key);
        let message = RuntimeMessage::new(key, message);

        if let Some(call_key) = call_key {
            self.pending_calls
                .insert(call_key, PendingCall { resolver: None });
        }
        self.commands.push_back(RuntimeCommand::Send {
            message,
            call: call_key,
        });

        call_key
    }

    pub(crate) fn push_incomming(&mut self, message: RuntimeMessage) {
        let message = RuntimeMessage::new(message.transaction_key, message.payload);

        if let Some(resolver) = self.recv_waiters.pop_front() {
            resolver.resolve(message);
        } else {
            self.incomming_msgs.push_back(message);
        }
    }

    pub(crate) fn recv(&mut self) -> InboundFuture {
        let (resolver, future) = promise();

        if let Some(message) = self.incomming_msgs.pop_front() {
            resolver.resolve(message);
        } else {
            self.recv_waiters.push_back(resolver);
        }

        future
    }
}

#[derive(Clone)]
pub struct SecsHandle {
    shared: SharedRuntime,
}

impl SecsHandle {
    pub(crate) fn new(shared: SharedRuntime) -> Self {
        Self { shared }
    }

    pub fn send(&self, message: Secs2Message) -> Option<TransactionKey> {
        self.shared().send(message)
    }

    pub fn send_with_key(&self, request_key: TransactionKey, message: Secs2Message) {
        self.shared().send_with_key(request_key, message);
    }

    pub fn reply(&self, key: TransactionKey, message: Secs2Message) {
        self.send_with_key(key, message);
    }

    pub fn request(&self, message: Secs2Message) -> RecvFuture {
        self.shared().request(message)
    }

    pub fn recv(&self) -> InboundFuture {
        self.shared().recv()
    }

    pub fn recv_call(&self, key: TransactionKey) -> RecvFuture {
        let mut shared = self.shared();
        let (resolver, future) = promise();

        let Some(pending) = shared.pending_calls.get_mut(&key) else {
            resolver.reject(CallError::UnknownToken);
            return future;
        };

        if pending.resolver.replace(resolver).is_some() {
            let (resolver, future) = promise();
            resolver.reject(CallError::UnknownToken);
            return future;
        }

        future
    }

    fn shared(&self) -> MutexGuard<'_, RuntimeShared> {
        self.shared
            .lock()
            .expect("runtime shared state mutex poisoned")
    }
}

pub type RecvFuture = PromiseFuture<Secs2Message, CallError>;
pub type InboundFuture = PromiseFuture<RuntimeMessage, CallError>;

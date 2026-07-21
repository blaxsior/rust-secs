use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use secs_ii::item::Secs2Variant;
use secs_ii::{FunctionId, Secs2Message, StreamId};

use crate::core::{CorrelationId, CorrelationIdSource, RuntimeMessage};
use crate::message::{MessageMachine, MessageRuntime, MessageRuntimeEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CallToken {
    pub correlation_id: CorrelationId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PendingCall {
    pub correlation_id: CorrelationId,
    pub stream: StreamId,
    pub function: FunctionId,
}

#[derive(Debug, Default)]
pub struct ExchangeTracker {
    pending_calls: BTreeMap<CorrelationId, PendingCall>,
}

impl ExchangeTracker {
    pub fn new() -> Self {
        Self {
            pending_calls: BTreeMap::new(),
        }
    }

    pub fn track_call(&mut self, msg: &RuntimeMessage) -> CallToken {
        let call = PendingCall {
            correlation_id: msg.correlation_id,
            stream: msg.stream(),
            function: msg.function(),
        };
        self.pending_calls.insert(msg.correlation_id, call);

        CallToken {
            correlation_id: msg.correlation_id,
        }
    }

    pub fn take_reply(&mut self, msg: &RuntimeMessage) -> Option<PendingCall> {
        if !msg.is_secondary() {
            return None;
        }

        self.pending_calls.remove(&msg.correlation_id)
    }

    pub fn is_reply_for(&self, token: CallToken, msg: &RuntimeMessage) -> bool {
        msg.is_secondary() && msg.correlation_id == token.correlation_id
    }

    pub fn has_pending(&self, correlation_id: CorrelationId) -> bool {
        self.pending_calls.contains_key(&correlation_id)
    }
}

pub trait Secs2MessageHandler {
    type Error;

    fn handle_primary(&mut self, primary: Secs2Message) -> Result<Option<Secs2Message>, Self::Error>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Secs2Route {
    pub stream: StreamId,
    pub function: FunctionId,
}

impl Secs2Route {
    pub const fn new(stream: StreamId, function: FunctionId) -> Self {
        Self { stream, function }
    }

    pub fn from_message(msg: &Secs2Message) -> Self {
        Self {
            stream: msg.stream,
            function: msg.function,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Secs2RequestContext {
    route: Secs2Route,
    need_reply: bool,
}

impl Secs2RequestContext {
    pub fn new(primary: &Secs2Message) -> Self {
        Self {
            route: Secs2Route::from_message(primary),
            need_reply: primary.need_reply,
        }
    }

    pub fn route(&self) -> Secs2Route {
        self.route
    }

    pub fn stream(&self) -> StreamId {
        self.route.stream
    }

    pub fn primary_function(&self) -> FunctionId {
        self.route.function
    }

    pub fn reply_function(&self) -> FunctionId {
        self.route.function.reply()
    }

    pub fn need_reply(&self) -> bool {
        self.need_reply
    }

    pub fn reply(&self, body: Option<Secs2Variant>) -> Secs2Message {
        Secs2Message::new(self.stream(), self.reply_function(), false, body)
    }
}

pub trait Secs2PrimaryHandler {
    type Error;

    fn route(&self) -> Secs2Route;

    fn handle(
        &mut self,
        ctx: Secs2RequestContext,
        primary: Secs2Message,
    ) -> Result<Option<Secs2Message>, Self::Error>;
}

pub trait Secs2DefaultHandler {
    type Error;

    fn handle_default(
        &mut self,
        ctx: Secs2RequestContext,
        primary: Secs2Message,
    ) -> Result<Option<Secs2Message>, Self::Error>;
}

pub struct IgnoreUnknownPrimary;

impl Secs2DefaultHandler for IgnoreUnknownPrimary {
    type Error = ();

    fn handle_default(
        &mut self,
        _ctx: Secs2RequestContext,
        _primary: Secs2Message,
    ) -> Result<Option<Secs2Message>, Self::Error> {
        Ok(None)
    }
}

pub struct Secs2Router<E> {
    handlers: Vec<Box<dyn Secs2PrimaryHandler<Error = E>>>,
    default_handler: Box<dyn Secs2DefaultHandler<Error = E>>,
}

impl<E> Secs2Router<E> {
    pub fn new<D>(default_handler: D) -> Self
    where
        D: Secs2DefaultHandler<Error = E> + 'static,
    {
        Self {
            handlers: Vec::new(),
            default_handler: Box::new(default_handler),
        }
    }

    pub fn add_handler<H>(&mut self, handler: H)
    where
        H: Secs2PrimaryHandler<Error = E> + 'static,
    {
        self.handlers.push(Box::new(handler));
    }

    pub fn with_handler<H>(mut self, handler: H) -> Self
    where
        H: Secs2PrimaryHandler<Error = E> + 'static,
    {
        self.add_handler(handler);
        self
    }

    pub fn set_default_handler<D>(&mut self, default_handler: D)
    where
        D: Secs2DefaultHandler<Error = E> + 'static,
    {
        self.default_handler = Box::new(default_handler);
    }

    pub fn handle_with_context(
        &mut self,
        ctx: Secs2RequestContext,
        primary: Secs2Message,
    ) -> Result<Option<Secs2Message>, E> {
        let route = ctx.route();

        for handler in self.handlers.iter_mut() {
            if handler.route() == route {
                return handler.handle(ctx, primary);
            }
        }

        self.default_handler.handle_default(ctx, primary)
    }
}

impl Secs2Router<()> {
    pub fn ignore_unknown() -> Self {
        Self::new(IgnoreUnknownPrimary)
    }
}

impl<E> Secs2MessageHandler for Secs2Router<E> {
    type Error = E;

    fn handle_primary(&mut self, primary: Secs2Message) -> Result<Option<Secs2Message>, Self::Error> {
        let ctx = Secs2RequestContext::new(&primary);
        self.handle_with_context(ctx, primary)
    }
}

pub enum Secs2RuntimeEvent<E> {
    Machine(E),
    Primary(RuntimeMessage),
    Reply(RuntimeMessage),
}

pub enum Secs2RuntimeError<L, H> {
    Lower(L),
    Handler(H),
    SecondaryWithoutPendingCall,
}

pub trait Secs2MessageLayer {
    type Error;
    type Event;

    fn send(&mut self, msg: RuntimeMessage) -> Result<(), Self::Error>;

    fn recv(&mut self) -> Option<RuntimeMessage>;

    fn poll_event(&mut self) -> Option<MessageRuntimeEvent<Self::Event>>;
}

impl<D, M, T> Secs2MessageLayer for MessageRuntime<D, M, T>
where
    M: MessageMachine,
{
    type Error = M::Error;
    type Event = M::Event;

    fn send(&mut self, msg: RuntimeMessage) -> Result<(), Self::Error> {
        MessageRuntime::send(self, msg)
    }

    fn recv(&mut self) -> Option<RuntimeMessage> {
        MessageRuntime::recv(self)
    }

    fn poll_event(&mut self) -> Option<MessageRuntimeEvent<Self::Event>> {
        MessageRuntime::poll_event(self)
    }
}

pub struct Secs2Runtime<L, C> {
    lower: L,
    correlation_ids: C,
    exchanges: ExchangeTracker,
}

impl<L, C> Secs2Runtime<L, C> {
    pub fn new(lower: L, correlation_ids: C) -> Self {
        Self {
            lower,
            correlation_ids,
            exchanges: ExchangeTracker::new(),
        }
    }

    pub fn lower(&self) -> &L {
        &self.lower
    }

    pub fn lower_mut(&mut self) -> &mut L {
        &mut self.lower
    }

    pub fn exchanges(&self) -> &ExchangeTracker {
        &self.exchanges
    }
}

impl<L, C> Secs2Runtime<L, C>
where
    C: CorrelationIdSource,
{
    pub fn make_primary(&mut self, payload: Secs2Message) -> RuntimeMessage {
        RuntimeMessage::new(self.correlation_ids.next_correlation_id(), payload)
    }
}

impl<L, C> Secs2Runtime<L, C>
where
    L: Secs2MessageLayer,
    C: CorrelationIdSource,
{
    pub fn start_call(&mut self, primary: Secs2Message) -> Result<CallToken, L::Error> {
        let msg = self.make_primary(primary);
        let token = self.exchanges.track_call(&msg);
        self.lower.send(msg)?;
        Ok(token)
    }

    pub fn poll_call<H>(
        &mut self,
        token: CallToken,
        handler: &mut H,
    ) -> Result<Option<Secs2Message>, Secs2RuntimeError<L::Error, H::Error>>
    where
        H: Secs2MessageHandler,
    {
        let Some(event) = self.poll_event(handler)? else {
            return Ok(None);
        };

        match event {
            Secs2RuntimeEvent::Reply(msg) if msg.correlation_id == token.correlation_id => {
                Ok(Some(msg.into_payload()))
            }
            _ => Ok(None),
        }
    }

    pub fn call<H>(
        &mut self,
        primary: Secs2Message,
        handler: &mut H,
    ) -> Result<Option<Secs2Message>, Secs2RuntimeError<L::Error, H::Error>>
    where
        H: Secs2MessageHandler,
    {
        let token = self.start_call(primary).map_err(Secs2RuntimeError::Lower)?;
        self.poll_call(token, handler)
    }

    pub fn handle<H>(
        &mut self,
        handler: &mut H,
    ) -> Result<Option<Secs2RuntimeEvent<L::Event>>, Secs2RuntimeError<L::Error, H::Error>>
    where
        H: Secs2MessageHandler,
    {
        self.poll_event(handler)
    }

    pub fn poll_event<H>(
        &mut self,
        handler: &mut H,
    ) -> Result<Option<Secs2RuntimeEvent<L::Event>>, Secs2RuntimeError<L::Error, H::Error>>
    where
        H: Secs2MessageHandler,
    {
        let Some(event) = self.lower.poll_event() else {
            return Ok(None);
        };

        match event {
            MessageRuntimeEvent::Machine(event) => Ok(Some(Secs2RuntimeEvent::Machine(event))),
            MessageRuntimeEvent::Message(msg) if msg.is_secondary() => {
                if self.exchanges.take_reply(&msg).is_none() {
                    return Err(Secs2RuntimeError::SecondaryWithoutPendingCall);
                }

                Ok(Some(Secs2RuntimeEvent::Reply(msg)))
            }
            MessageRuntimeEvent::Message(msg) => {
                let correlation_id = msg.correlation_id;
                let reply = handler
                    .handle_primary(msg.into_payload())
                    .map_err(Secs2RuntimeError::Handler)?;

                if let Some(reply) = reply {
                    self.lower
                        .send(RuntimeMessage::new(correlation_id, reply))
                        .map_err(Secs2RuntimeError::Lower)?;
                }

                Ok(None)
            }
        }
    }
}

use alloc::{
    boxed::Box,
    collections::BTreeMap,
    string::String,
    vec::Vec,
};
use core::{future::Future, pin::Pin};

use secs_ii::{FunctionId, StreamId};
use secs_runtime::SecsHandle;
use secs_runtime_core::RuntimeMessage;

use crate::{SecsContext, SecsHandlerError};

pub type BoxSecsRouteFuture<'a> =
    Pin<Box<dyn Future<Output = Result<(), SecsHandlerError>> + 'a>>;
pub type BoxSecsActionFuture<'a> =
    Pin<Box<dyn Future<Output = Result<(), SecsHandlerError>> + 'a>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SecsRoute {
    pub stream: StreamId,
    pub function: FunctionId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecsMatcher {
    Exact(SecsRoute),
    Stream(StreamId),
    Any,
}

impl SecsMatcher {
    pub fn exact(stream: StreamId, function: FunctionId) -> Self {
        Self::Exact(SecsRoute::new(stream, function))
    }

    pub fn stream(stream: StreamId) -> Self {
        Self::Stream(stream)
    }

    pub fn any() -> Self {
        Self::Any
    }

    pub fn matches(&self, message: &RuntimeMessage) -> bool {
        match self {
            Self::Exact(route) => *route == SecsRoute::from_message(message),
            Self::Stream(stream) => *stream == message.stream(),
            Self::Any => true,
        }
    }
}

impl SecsRoute {
    pub fn new(stream: StreamId, function: FunctionId) -> Self {
        Self { stream, function }
    }

    pub fn from_message(message: &RuntimeMessage) -> Self {
        Self::new(message.stream(), message.function())
    }
}

pub trait SecsRouteHandler {
    fn handle<'a>(
        &'a mut self,
        ctx: SecsContext,
        message: RuntimeMessage,
    ) -> BoxSecsRouteFuture<'a>;
}

impl<F, Fut> SecsRouteHandler for F
where
    F: FnMut(SecsContext, RuntimeMessage) -> Fut,
    Fut: Future<Output = Result<(), SecsHandlerError>> + 'static,
{
    fn handle<'a>(
        &'a mut self,
        ctx: SecsContext,
        message: RuntimeMessage,
    ) -> BoxSecsRouteFuture<'a> {
        Box::pin(self(ctx, message))
    }
}

pub trait SecsAction {
    fn call<'a>(&'a mut self, ctx: SecsContext) -> BoxSecsActionFuture<'a>;
}

impl<F, Fut> SecsAction for F
where
    F: FnMut(SecsContext) -> Fut,
    Fut: Future<Output = Result<(), SecsHandlerError>> + 'static,
{
    fn call<'a>(&'a mut self, ctx: SecsContext) -> BoxSecsActionFuture<'a> {
        Box::pin(self(ctx))
    }
}

pub struct SecsHandler {
    routes: Vec<SecsRouteEntry>,
    actions: BTreeMap<String, Box<dyn SecsAction>>,
}

struct SecsRouteEntry {
    matcher: SecsMatcher,
    handler: Box<dyn SecsRouteHandler>,
}

impl SecsHandler {
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            actions: BTreeMap::new(),
        }
    }

    pub fn on<F, Fut>(&mut self, matcher: SecsMatcher, handler: F)
    where
        F: FnMut(SecsContext, RuntimeMessage) -> Fut + 'static,
        Fut: Future<Output = Result<(), SecsHandlerError>> + 'static,
    {
        self.on_handler(matcher, handler);
    }

    pub fn on_handler<H>(&mut self, matcher: SecsMatcher, handler: H)
    where
        H: SecsRouteHandler + 'static,
    {
        self.routes.push(SecsRouteEntry {
            matcher,
            handler: Box::new(handler),
        });
    }

    pub fn action<F, Fut>(&mut self, name: impl Into<String>, action: F)
    where
        F: FnMut(SecsContext) -> Fut + 'static,
        Fut: Future<Output = Result<(), SecsHandlerError>> + 'static,
    {
        self.action_handler(name, action);
    }

    pub fn action_handler<A>(&mut self, name: impl Into<String>, action: A)
    where
        A: SecsAction + 'static,
    {
        self.actions.insert(name.into(), Box::new(action));
    }

    pub async fn dispatch(
        &mut self,
        ctx: SecsContext,
        message: RuntimeMessage,
    ) -> Result<(), SecsHandlerError> {
        let Some(entry) = self
            .routes
            .iter_mut()
            .find(|entry| entry.matcher.matches(&message))
        else {
            return Err(SecsHandlerError::RouteNotFound);
        };

        entry.handler.handle(ctx, message).await
    }

    pub async fn call(
        &mut self,
        ctx: SecsContext,
        name: &str,
    ) -> Result<(), SecsHandlerError> {
        let Some(action) = self.actions.get_mut(name) else {
            return Err(SecsHandlerError::ActionNotFound);
        };

        action.call(ctx).await
    }

    pub async fn run(&mut self, handle: SecsHandle) -> Result<(), SecsHandlerError> {
        loop {
            let message = handle.recv().await?;
            self.dispatch(SecsContext::new(handle.clone()), message)
                .await?;
        }
    }
}

impl Default for SecsHandler {
    fn default() -> Self {
        Self::new()
    }
}

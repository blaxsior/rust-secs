use std::{
    boxed::Box,
    collections::BTreeMap,
    string::String,
    sync::Arc,
    vec::Vec,
};
use core::{future::Future, pin::Pin};

use secs_ii::{FunctionId, StreamId};
use secs_runtime::SecsHandle;
use secs_runtime_core::RuntimeMessage;

use crate::{SecsContext, SecsHandlerError};

pub type BoxSecsRouteFuture<'a> =
    Pin<Box<dyn Future<Output = Result<(), SecsHandlerError>> + Send + 'a>>;
pub type BoxSecsActionFuture<'a> =
    Pin<Box<dyn Future<Output = Result<(), SecsHandlerError>> + Send + 'a>>;

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

impl From<SecsRoute> for SecsMatcher {
    fn from(value: SecsRoute) -> Self {
        Self::Exact(value)
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

pub trait SecsRouteHandler: Send + Sync {
    fn handle<'a>(
        &'a self,
        ctx: SecsContext,
        message: RuntimeMessage,
    ) -> BoxSecsRouteFuture<'a>;
}

impl<F, Fut> SecsRouteHandler for F
where
    F: Fn(SecsContext, RuntimeMessage) -> Fut + Send + Sync,
    Fut: Future<Output = Result<(), SecsHandlerError>> + Send + 'static,
{
    fn handle<'a>(
        &'a self,
        ctx: SecsContext,
        message: RuntimeMessage,
    ) -> BoxSecsRouteFuture<'a> {
        Box::pin(self(ctx, message))
    }
}

pub trait SecsAction: Send + Sync {
    fn call<'a>(&'a self, ctx: SecsContext) -> BoxSecsActionFuture<'a>;
}

impl<F, Fut> SecsAction for F
where
    F: Fn(SecsContext) -> Fut + Send + Sync,
    Fut: Future<Output = Result<(), SecsHandlerError>> + Send + 'static,
{
    fn call<'a>(&'a self, ctx: SecsContext) -> BoxSecsActionFuture<'a> {
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

struct ArcSecsRouteHandler<H>
where
    H: SecsRouteHandler + ?Sized,
{
    inner: Arc<H>,
}

impl<H> ArcSecsRouteHandler<H>
where
    H: SecsRouteHandler + ?Sized,
{
    fn new(inner: Arc<H>) -> Self {
        Self { inner }
    }
}

impl<H> SecsRouteHandler for ArcSecsRouteHandler<H>
where
    H: SecsRouteHandler + ?Sized,
{
    fn handle<'a>(
        &'a self,
        ctx: SecsContext,
        message: RuntimeMessage,
    ) -> BoxSecsRouteFuture<'a> {
        self.inner.handle(ctx, message)
    }
}

impl SecsHandler {
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            actions: BTreeMap::new(),
        }
    }

    pub fn on<F, Fut>(&mut self, matcher: impl Into<SecsMatcher>, handler: F)
    where
        F: Fn(SecsContext, RuntimeMessage) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), SecsHandlerError>> + Send + 'static,
    {
        self.on_handler(matcher, handler);
    }

    pub fn on_handler<H>(&mut self, matcher: impl Into<SecsMatcher>, handler: H)
    where
        H: SecsRouteHandler + 'static,
    {
        self.routes.push(SecsRouteEntry {
            matcher: matcher.into(),
            handler: Box::new(handler),
        });
    }

    pub fn on_component<H>(&mut self, matcher: impl Into<SecsMatcher>, handler: Arc<H>)
    where
        H: SecsRouteHandler + ?Sized + 'static,
    {
        self.on_handler(matcher, ArcSecsRouteHandler::new(handler));
    }

    pub fn action<F, Fut>(&mut self, name: impl Into<String>, action: F)
    where
        F: Fn(SecsContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), SecsHandlerError>> + Send + 'static,
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
            .iter()
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
        let Some(action) = self.actions.get(name) else {
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

// #[macro_export]
// macro_rules! secs_handler {
//     ($route:ident, $handler:ty) => {
//         $crate::paste::paste! {
//             pub trait [<$route Handler>]: $crate::SecsRouteHandler + $crate::shaku::Interface {}

//             impl [<$route Handler>] for $handler {}
//         }
//     };
//     ($route:ident => $handler:ty) => {
//         $crate::secs_handler!($route, $handler);
//     };
//     ($visibility:vis $interface:ident => $handler:ty) => {
//         $visibility trait $interface: $crate::SecsRouteHandler + $crate::shaku::Interface {}

//         impl $interface for $handler {}
//     };
// }

// #[macro_export]
// macro_rules! secs_handle {
//     ($registry:expr, $module:expr; $($matcher:expr => $interface:path),* $(,)?) => {
//         $(
//             {
//                 let handler: std::sync::Arc<dyn $interface> =
//                     $crate::shaku::HasComponent::<dyn $interface>::resolve(&$module);
//                 $registry.on_component($matcher, handler);
//             }
//         )*
//     };
// }

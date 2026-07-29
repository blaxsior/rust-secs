use alloc::boxed::Box;
use core::{future::Future, pin::Pin};

use crate::{error::HandlerError, scenario::ScenarioContext};

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

pub trait SecsScenario {
    fn run(self, ctx: ScenarioContext) -> impl Future<Output = Result<(), HandlerError>> + 'static;
}

pub(crate) trait BoxedSecsScenario {
    fn run_boxed(
        self: Box<Self>,
        ctx: ScenarioContext,
    ) -> BoxFuture<'static, Result<(), HandlerError>>;
}

impl<T> BoxedSecsScenario for T
where
    T: SecsScenario + 'static,
{
    fn run_boxed(
        self: Box<Self>,
        ctx: ScenarioContext,
    ) -> BoxFuture<'static, Result<(), HandlerError>> {
        Box::pin((*self).run(ctx))
    }
}

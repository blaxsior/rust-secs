use alloc::boxed::Box;
use core::{future::Future, pin::Pin};

use crate::{error::HandlerError, scenario::ScenarioContext};

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

pub trait SecsScenario {
    fn run(self: Box<Self>, ctx: ScenarioContext) -> BoxFuture<'static, Result<(), HandlerError>>;
}

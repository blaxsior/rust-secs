use std::{cell::RefCell, convert::Infallible, rc::Rc};

use secs_ii::{FunctionId, Secs2Message, StreamId};
use secs_runtime::{
    ScenarioTaskOutput, SecsRuntime, SecsRuntimeError, SecsRuntimeRoute, SecsScenario, SecsService,
    TimeoutConfig,
};
use secs_runtime_core::SystemByteSource;
use secs_transport::transport::hsms::{config::HsmsTransportConfig, protocol::HsmsTransport};

use crate::{WebDataSource, WebDataSourceHandle, WebSecsTimer, WebTaskRunner};

type InnerRuntime = SecsRuntime<WebSecsTimer, WebTaskRunner<ScenarioTaskOutput>>;

pub type WebRuntimeError = SecsRuntimeError<Infallible>;

pub struct WebRuntime {
    inner: Rc<RefCell<InnerRuntime>>,
    data_source_handle: WebDataSourceHandle,
}

impl WebRuntime {
    pub fn new(config: &HsmsTransportConfig, data_source: WebDataSource) -> Self {
        let data_source_handle = data_source.handle();
        let transport = HsmsTransport::new(
            config,
            Box::new(data_source),
            SystemByteSource::with_range(1, 1, 1023),
        );

        Self {
            inner: Rc::new(RefCell::new(SecsRuntime::with_task_runner(
                transport,
                WebSecsTimer::new(),
                SystemByteSource::with_range(1024, 1024, u32::MAX),
                hsms_timeout_config(config),
                WebTaskRunner::new(),
            ))),
            data_source_handle,
        }
    }

    pub fn start(&mut self) -> Result<(), WebRuntimeError> {
        self.inner.borrow_mut().start()
    }

    pub fn tick(&mut self) -> Result<(), WebRuntimeError> {
        self.tick_inner()
    }

    pub fn register_service<H>(&mut self, stream: StreamId, function: FunctionId, service: H)
    where
        H: SecsService + 'static,
    {
        self.inner
            .borrow_mut()
            .register_service(stream, function, service);
    }

    pub fn start_scenario<H>(&mut self, scenario: H) -> Result<(), WebRuntimeError>
    where
        H: SecsScenario + 'static,
    {
        self.inner.borrow_mut().start_scenario(scenario)
    }

    pub fn poll_received(&mut self) -> Option<Secs2Message> {
        self.inner.borrow_mut().poll_received()
    }

    pub fn route(stream: StreamId, function: FunctionId) -> SecsRuntimeRoute {
        SecsRuntimeRoute::new(stream, function)
    }

    pub fn data_source_handle(&self) -> WebDataSourceHandle {
        self.data_source_handle.clone()
    }

    fn tick_inner(&mut self) -> Result<(), WebRuntimeError> {
        tick_runtime(&self.inner)
    }
}

fn tick_runtime(runtime: &Rc<RefCell<InnerRuntime>>) -> Result<(), WebRuntimeError> {
    let mut runtime = runtime.borrow_mut();
    runtime.tick()?;
    log_unrouted_messages(&mut runtime);
    Ok(())
}

fn log_unrouted_messages(runtime: &mut InnerRuntime) {
    while let Some(message) = runtime.poll_received() {
        log::warn!(
            "received unrouted message: S{}F{}, need_reply={}",
            message.stream.0,
            message.function.0,
            message.need_reply
        );
    }
}

pub fn hsms_timeout_config(config: &HsmsTransportConfig) -> TimeoutConfig<f64> {
    TimeoutConfig {
        t1: config.t3_timeout.as_millis() as f64,
        t2: config.t3_timeout.as_millis() as f64,
        t3: config.t3_timeout.as_millis() as f64,
        t4: config.t8_timeout.as_millis() as f64,
        t5: config.t5_timeout.as_millis() as f64,
        t6: config.t6_timeout.as_millis() as f64,
        t7: config.t7_timeout.as_millis() as f64,
        t8: config.t8_timeout.as_millis() as f64,
    }
}

use std::{
    convert::Infallible,
    sync::{Arc, Mutex, MutexGuard},
};

use secs_runtime::{SecsHandle, SecsRuntime, SecsRuntimeError, TimeoutConfig};
use secs_runtime_core::SystemByteSource;
use secs_transport::transport::hsms::{config::HsmsTransportConfig, protocol::HsmsTransport};

use crate::{WebDataSource, WebDataSourceHandle, WebSecsTimer};

type InnerRuntime = SecsRuntime<WebSecsTimer, HsmsTransport>;

pub type WebRuntimeError = SecsRuntimeError<Infallible>;

pub struct WebRuntime {
    inner: Arc<Mutex<InnerRuntime>>,
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
            inner: Arc::new(Mutex::new(SecsRuntime::new(
                transport,
                WebSecsTimer::new(),
                SystemByteSource::with_range(1024, 1024, u32::MAX),
                hsms_timeout_config(config),
            ))),
            data_source_handle,
        }
    }

    pub fn start(&mut self) -> Result<(), WebRuntimeError> {
        self.inner().start()
    }

    pub fn tick(&mut self) -> Result<(), WebRuntimeError> {
        self.tick_inner()
    }

    pub fn handle(&self) -> SecsHandle {
        self.inner().handle()
    }

    pub fn data_source_handle(&self) -> WebDataSourceHandle {
        self.data_source_handle.clone()
    }

    fn tick_inner(&mut self) -> Result<(), WebRuntimeError> {
        self.inner().tick()
    }

    fn inner(&self) -> MutexGuard<'_, InnerRuntime> {
        self.inner.lock().expect("web runtime mutex poisoned")
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

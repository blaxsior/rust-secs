use core::{net::SocketAddr, time::Duration};

use js_sys::{Function, Uint8Array};
use log::LevelFilter;
use secs_common::{ConnectionRole, SessionId};
use secs_transport::transport::hsms::config::HsmsTransportConfig;
use wasm_bindgen::prelude::*;

use crate::{WebDataSource, WebDataSourceHandle, WebRuntime, init_logger_with_callback};

#[wasm_bindgen]
pub struct JsWebRuntime {
    inner: WebRuntime,
    data_source: WebDataSourceHandle,
}

#[wasm_bindgen]
impl JsWebRuntime {
    #[wasm_bindgen(constructor)]
    pub fn new(
        session_id: u16,
        role: &str,
        on_write: Function,
        on_open: Option<Function>,
        on_close: Option<Function>,
        on_read_request: Option<Function>,
    ) -> Result<JsWebRuntime, JsValue> {
        let config = default_hsms_config(session_id, parse_role(role)?);
        let data_source =
            WebDataSource::with_callbacks(on_write, on_open, on_close, on_read_request);
        let runtime = WebRuntime::new(&config, data_source);
        let data_source = runtime.data_source_handle();

        Ok(Self {
            inner: runtime,
            data_source,
        })
    }

    pub fn start(&mut self) -> Result<(), JsValue> {
        self.inner
            .start()
            .map_err(|error| JsValue::from_str(&format!("{:?}", error)))
    }

    pub fn tick(&mut self) -> Result<(), JsValue> {
        self.inner
            .tick()
            .map_err(|error| JsValue::from_str(&format!("{:?}", error)))
    }

    #[wasm_bindgen(js_name = markOpen)]
    pub fn mark_open(&self) {
        self.data_source.open();
    }

    #[wasm_bindgen(js_name = markClosed)]
    pub fn mark_closed(&self) {
        self.data_source.close();
    }

    #[wasm_bindgen(js_name = markFailed)]
    pub fn mark_failed(&self) {
        self.data_source.fail();
    }

    #[wasm_bindgen(js_name = isOpen)]
    pub fn is_open(&self) -> bool {
        self.data_source.is_open()
    }

    #[wasm_bindgen(js_name = hasError)]
    pub fn has_error(&self) -> bool {
        self.data_source.has_error()
    }

    #[wasm_bindgen(js_name = pendingReadLength)]
    pub fn pending_read_len(&self) -> usize {
        self.data_source.pending_read_len()
    }

    #[wasm_bindgen(js_name = dataSourceState)]
    pub fn data_source_state(&self) -> String {
        if self.data_source.has_error() {
            "error".to_string()
        } else if self.data_source.is_open() {
            "open".to_string()
        } else {
            "closed".to_string()
        }
    }

    #[wasm_bindgen(js_name = pushReadBytes)]
    pub fn push_read_bytes(&self, bytes: Uint8Array) {
        self.data_source.push_read_u8_array(bytes);
    }
}

#[wasm_bindgen]
pub fn init_web_logger(level: &str, callback: Function) -> Result<(), JsValue> {
    init_logger_with_callback(parse_level(level), callback)
        .map_err(|error| JsValue::from_str(&format!("{:?}", error)))
}

fn parse_role(role: &str) -> Result<ConnectionRole, JsValue> {
    match role.to_ascii_lowercase().as_str() {
        "active" => Ok(ConnectionRole::Active),
        "passive" => Ok(ConnectionRole::Passive),
        _ => Err(JsValue::from_str("role must be 'active' or 'passive'")),
    }
}

fn parse_level(level: &str) -> LevelFilter {
    match level.to_ascii_lowercase().as_str() {
        "off" => LevelFilter::Off,
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Info,
    }
}

fn default_hsms_config(session_id: u16, connection_mode: ConnectionRole) -> HsmsTransportConfig {
    HsmsTransportConfig {
        session_id: SessionId(session_id),
        connection_mode,
        t3_timeout: Duration::from_secs(45),
        t5_timeout: Duration::from_secs(10),
        t6_timeout: Duration::from_secs(5),
        t7_timeout: Duration::from_secs(10),
        t8_timeout: Duration::from_secs(5),
        local_addr: dummy_addr(),
        remote_addr: dummy_addr(),
    }
}

fn dummy_addr() -> SocketAddr {
    SocketAddr::from(([0, 0, 0, 0], 0))
}

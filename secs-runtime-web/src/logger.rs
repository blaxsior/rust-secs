use std::cell::RefCell;

use js_sys::Function;
use log::{LevelFilter, Log, Metadata, Record, SetLoggerError};
use wasm_bindgen::JsValue;

static LOGGER: WebLogger = WebLogger;

pub struct WebLogger;

thread_local! {
    static LOG_CALLBACK: RefCell<Option<Function>> = const { RefCell::new(None) };
}

impl WebLogger {
    pub fn init(level: LevelFilter) -> Result<(), SetLoggerError> {
        log::set_logger(&LOGGER)?;
        log::set_max_level(level);
        Ok(())
    }

    pub fn init_with_callback(
        level: LevelFilter,
        callback: Function,
    ) -> Result<(), SetLoggerError> {
        set_log_callback(callback);
        match Self::init(level) {
            Ok(()) => Ok(()),
            Err(error) => {
                log::set_max_level(level);
                Err(error)
            }
        }
    }
}

impl Log for WebLogger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let level = record.level();
        let message = format!("[{}] {}", level, record.args());
        let _ = LOG_CALLBACK.with(|callback| {
            let Some(callback) = callback.borrow().as_ref().cloned() else {
                return false;
            };

            let _ = callback.call2(
                &JsValue::NULL,
                &JsValue::from_str(level.as_str()),
                &JsValue::from_str(&message),
            );
            true
        });
    }

    fn flush(&self) {}
}

pub fn set_log_callback(callback: Function) {
    LOG_CALLBACK.with(|slot| {
        *slot.borrow_mut() = Some(callback);
    });
}

pub fn init_logger(level: LevelFilter) -> Result<(), SetLoggerError> {
    WebLogger::init(level)
}

pub fn init_logger_with_callback(
    level: LevelFilter,
    callback: Function,
) -> Result<(), SetLoggerError> {
    WebLogger::init_with_callback(level, callback)
}

use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use js_sys::{Function, Uint8Array};
use secs_runtime_core::{ByteDataSource, ByteDataSourceError};
use wasm_bindgen::JsValue;

#[derive(Default)]
struct WebDataSourceState {
    incoming: VecDeque<u8>,
    is_open: bool,
    has_error: bool,
}

#[derive(Clone)]
pub struct WebDataSourceHandle {
    state: Rc<RefCell<WebDataSourceState>>,
}

impl WebDataSourceHandle {
    pub fn open(&self) {
        let mut state = self.state.borrow_mut();
        state.is_open = true;
        state.has_error = false;
    }

    pub fn close(&self) {
        let mut state = self.state.borrow_mut();
        state.is_open = false;
        state.incoming.clear();
    }

    pub fn fail(&self) {
        self.state.borrow_mut().has_error = true;
    }

    pub fn is_open(&self) -> bool {
        self.state.borrow().is_open
    }

    pub fn has_error(&self) -> bool {
        self.state.borrow().has_error
    }

    pub fn pending_read_len(&self) -> usize {
        self.state.borrow().incoming.len()
    }

    pub fn push_read_bytes(&self, bytes: &[u8]) {
        self.state.borrow_mut().incoming.extend(bytes);
    }

    pub fn push_read_u8_array(&self, bytes: Uint8Array) {
        self.push_read_bytes(&bytes.to_vec());
    }
}

pub struct WebDataSource {
    state: Rc<RefCell<WebDataSourceState>>,
    on_open: Option<Function>,
    on_close: Option<Function>,
    on_read_request: Option<Function>,
    on_write: Function,
}

impl WebDataSource {
    pub fn new(on_write: Function) -> Self {
        Self {
            state: Rc::new(RefCell::new(WebDataSourceState::default())),
            on_open: None,
            on_close: None,
            on_read_request: None,
            on_write,
        }
    }

    pub fn with_callbacks(
        on_write: Function,
        on_open: Option<Function>,
        on_close: Option<Function>,
        on_read_request: Option<Function>,
    ) -> Self {
        Self {
            state: Rc::new(RefCell::new(WebDataSourceState::default())),
            on_open,
            on_close,
            on_read_request,
            on_write,
        }
    }

    pub fn handle(&self) -> WebDataSourceHandle {
        WebDataSourceHandle {
            state: self.state.clone(),
        }
    }

    fn call_optional(callback: &Option<Function>) -> Result<(), ByteDataSourceError> {
        let Some(callback) = callback else {
            return Ok(());
        };

        callback
            .call0(&JsValue::NULL)
            .map(|_| ())
            .map_err(|_| ByteDataSourceError::WriteFailed)
    }
}

impl ByteDataSource for WebDataSource {
    fn open(&mut self) -> Result<(), ByteDataSourceError> {
        Self::call_optional(&self.on_open).map_err(|_| ByteDataSourceError::OpenFailed)
    }

    fn close(&mut self) -> Result<(), ByteDataSourceError> {
        Self::call_optional(&self.on_close).map_err(|_| ByteDataSourceError::CloseFailed)?;
        self.state.borrow_mut().is_open = false;
        Ok(())
    }

    fn is_open(&self) -> bool {
        self.state.borrow().is_open
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ByteDataSourceError> {
        if self.state.borrow().has_error {
            return Err(ByteDataSourceError::ReadFailed);
        }

        Self::call_optional(&self.on_read_request).map_err(|_| ByteDataSourceError::ReadFailed)?;

        let mut state = self.state.borrow_mut();
        if !state.is_open {
            return Err(ByteDataSourceError::WouldBlock);
        }

        let len = buf.len().min(state.incoming.len());
        for slot in buf.iter_mut().take(len) {
            *slot = state.incoming.pop_front().expect("incoming length checked");
        }

        Ok(len)
    }

    fn write(&mut self, bytes: &[u8]) -> Result<(), ByteDataSourceError> {
        if !self.state.borrow().is_open {
            return Err(ByteDataSourceError::WouldBlock);
        }

        let bytes = Uint8Array::from(bytes);
        self.on_write
            .call1(&JsValue::NULL, bytes.as_ref())
            .map(|_| ())
            .map_err(|_| ByteDataSourceError::WriteFailed)
    }
}

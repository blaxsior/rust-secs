use sansio::Protocol;
use secs_transport::transport::{
    SecsTimeoutUnit, TimeoutTicket,
    secs1::{
        block::Secs1Block,
        config::Secs1TransportConfig,
        protocol::block_transfer::Secs1BlockTransferMachine,
    },
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct JsSecs1BlockTransfer {
    inner: Secs1BlockTransferMachine,
    pending_timeouts: Vec<TimeoutTicket>,
}

#[wasm_bindgen]
impl JsSecs1BlockTransfer {
    #[wasm_bindgen(constructor)]
    pub fn new(config_json: &str) -> Result<Self, JsValue> {
        let config: Secs1TransportConfig =
            serde_json::from_str(config_json).map_err(to_js_error)?;
        Ok(Self {
            inner: Secs1BlockTransferMachine::new(&config),
            pending_timeouts: Vec::new(),
        })
    }

    pub fn read(&mut self, bytes: Vec<u8>) -> Result<(), JsValue> {
        self.inner.handle_read(&bytes).map_err(to_js_error)
    }

    pub fn write(&mut self, block_json: &str) -> Result<(), JsValue> {
        let block: Secs1Block = serde_json::from_str(block_json).map_err(to_js_error)?;
        self.inner.handle_write(block).map_err(to_js_error)
    }


    pub fn timeout(&mut self, key: &str) -> Result<(), JsValue> {
        let key: Secs1TimeoutKey = serde_json::from_str(key).map_err(to_js_error)?;
        let Some(index) = self
            .pending_timeouts
            .iter()
            .position(|ticket| key.matches(*ticket))
        else {
            return Err(JsValue::from_str("unknown or already consumed timeout key"));
        };

        let ticket = self.pending_timeouts.remove(index);
        self.inner.handle_timeout(ticket).map_err(to_js_error)
    }

    pub fn poll_write(&mut self) -> Option<Vec<u8>> {
        self.inner.poll_write()
    }

    pub fn poll_read(&mut self) -> Result<Option<String>, JsValue> {
        self.inner
            .poll_read()
            .map(|block| serde_json::to_string(&block).map_err(to_js_error))
            .transpose()
    }


    pub fn poll_timeout(&mut self) -> Option<String> {
        self.inner.poll_timeout().map(|ticket| {
            let key = serde_json::to_string(&Secs1TimeoutKey::from(ticket))
                .expect("timeout key must be serializable");
            self.pending_timeouts.push(ticket);
            key
        })
    }

    pub fn poll_event(&mut self) -> Result<Option<String>, JsValue> {
        Ok(self.inner.poll_event().map(|event| format!("{event:?}")))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct Secs1TimeoutKey {
    id: u64,
    unit: String,
}

impl Secs1TimeoutKey {
    fn matches(&self, ticket: TimeoutTicket) -> bool {
        self.id == ticket.id.0 && self.unit == secs1_timeout_name(ticket.timeout)
    }
}

impl From<TimeoutTicket> for Secs1TimeoutKey {
    fn from(ticket: TimeoutTicket) -> Self {
        Self {
            id: ticket.id.0,
            unit: secs1_timeout_name(ticket.timeout).to_string(),
        }
    }
}

fn secs1_timeout_name(unit: SecsTimeoutUnit) -> &'static str {
    match unit {
        SecsTimeoutUnit::T1 => "t1",
        SecsTimeoutUnit::T2 => "t2",
        _ => "unsupported",
    }
}

fn to_js_error(error: impl core::fmt::Debug) -> JsValue {
    JsValue::from_str(&format!("{error:?}"))
}

use secs_ii::convert::secs2::{parse, serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn decode_secs2(bytes: Vec<u8>) -> Result<String, JsValue> {
    match parse::parse(&bytes) {
        Ok(data) => {
            Ok(serde_json::to_string(&data)
                .map_err(|err| JsValue::from_str(&format!("{:?}", err)))?)
        }
        Err(e) => Err(JsValue::from_str(&e)),
    }
}

#[wasm_bindgen]
pub fn encode_secs2(json: &str) -> Result<Vec<u8>, JsValue> {
    let variant = serde_json::from_str(json).map_err(|err| JsValue::from_str(&err.to_string()))?;
    serialize::serialize(&variant).map_err(|err| JsValue::from_str(&format!("{:?}", err)))
}

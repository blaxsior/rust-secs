use alloc::{borrow::ToOwned, collections::BTreeMap, string::String, vec::Vec};
use core::fmt;

use secs_ii::convert::secs2::serialize::Encode;
use secs_ii::item::{Secs2FormatCode, Secs2Variant};
use serde::{Deserialize, Serialize};

use crate::{
    NoopValueDataRepository, NoopValueSpecRepository, SecsModelError, StoreError,
    ValueDataRepository, ValueSpecRepository,
};

#[repr(transparent)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ValueId(String);

impl ValueId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Debug for ValueId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("VID: ").field(&self.0).finish()
    }
}

impl From<String> for ValueId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for ValueId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ValueSpec {
    pub id: ValueId,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    pub format: Secs2FormatCode,
    #[serde(default = "crate::domain::default_policy::persistent")]
    pub persistent: bool,
    #[serde(default = "crate::domain::default_policy::readonly")]
    pub readonly: bool,
}

impl ValueSpec {
    pub fn new(id: ValueId, format: Secs2FormatCode) -> Self {
        Self {
            id,
            name: None,
            description: None,
            format,
            persistent: false,
            readonly: false,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn is_persistent(mut self) -> Self {
        self.persistent = true;
        self
    }

    pub fn is_readonly(mut self) -> Self {
        self.readonly = true;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ValueData {
    pub id: ValueId,
    pub encoded: String,
}

impl ValueData {
    pub fn new(id: ValueId, encoded: String) -> Self {
        Self { id, encoded }
    }
}

#[derive(Debug)]
pub struct ValueDictionary<S, D> {
    specs: BTreeMap<ValueId, ValueSpec>,
    values: BTreeMap<ValueId, Secs2Variant>,
    spec_repository: S,
    data_repository: D,
}

impl<S, D> ValueDictionary<S, D>
where
    S: ValueSpecRepository,
    D: ValueDataRepository,
{
    pub fn with_store(spec_repository: S, data_repository: D) -> Result<Self, StoreError> {
        let mut dictionary = Self {
            specs: BTreeMap::new(),
            values: BTreeMap::new(),
            spec_repository,
            data_repository,
        };

        for spec in dictionary.spec_repository.load_all()? {
            dictionary.insert_spec(spec);
        }

        for data in dictionary.data_repository.load_all()? {
            dictionary.insert_data(data);
        }

        Ok(dictionary)
    }

    fn insert_spec(&mut self, spec: ValueSpec) {
        self.specs.insert(spec.id.clone(), spec);
    }

    fn insert_data(&mut self, data: ValueData) {
        if self.specs.contains_key(&data.id) {
            let Ok(bytes) = decode_hex(&data.encoded) else {
                log::error!("failed to decode value data hex: {:?}", data.id);
                return;
            };

            match Secs2Variant::try_from(bytes.as_slice()) {
                Ok(value) => {
                    self.values.insert(data.id, value);
                }
                Err(err) => {
                    log::error!("failed to decode value data {:?}: {:?}", data.id, err);
                }
            }
        }
    }

    pub fn spec(&self, id: &ValueId) -> Option<&ValueSpec> {
        self.specs.get(id)
    }

    pub fn read(&self, id: &ValueId) -> Result<Option<&Secs2Variant>, SecsModelError> {
        self.spec(id)
            .ok_or_else(|| SecsModelError::UnknownValue(id.clone()))?;

        Ok(self.values.get(id))
    }

    pub fn write(&mut self, id: &ValueId, value: Secs2Variant) -> Result<(), SecsModelError> {
        let spec = self
            .spec(id)
            .ok_or_else(|| SecsModelError::UnknownValue(id.clone()))?;

        if spec.readonly {
            log::warn!("skip value write because value is readonly: {:?}", id);
            return Err(SecsModelError::ReadOnlyValue(id.clone()));
        }

        let actual = value.format_code();
        if spec.format != actual {
            log::warn!(
                "skip value write because value format is invalid: {:?}, expected={:?}, actual={:?}",
                id,
                spec.format,
                actual
            );
            return Err(SecsModelError::InvalidValueFormat {
                id: id.clone(),
                expected: spec.format,
                actual,
            });
        }

        let mut bytes = Vec::new();
        if let Err(err) = value.encode(&mut bytes) {
            log::warn!("skip value write because value encode failed: {:?}, {:?}", id, err);
            return Err(SecsModelError::EncodeValue(id.clone()));
        }

        let data = ValueData::new(id.clone(), encode_hex(&bytes));
        if spec.persistent {
            if let Err(err) = self.data_repository.save(&data) {
                log::error!("failed to save data on repository {:?}", err);
            }
        }
        self.values.insert(data.id, value);
        Ok(())
    }

    pub fn remove(&mut self, id: &ValueId) -> Result<(), SecsModelError> {
        let spec = self
            .spec(id)
            .ok_or_else(|| SecsModelError::UnknownValue(id.clone()))?;

        if spec.readonly {
            log::warn!("skip value write because value is readonly: {:?}", id);
            return Err(SecsModelError::ReadOnlyValue(id.clone()));
        }

        if spec.persistent {
            if let Err(err) = self.data_repository.remove(id) {
                log::error!("failed to remove data from repository {:?}", err);
            }
        }
        self.values.remove(id);
        Ok(())
    }
}

impl ValueDictionary<NoopValueSpecRepository, NoopValueDataRepository> {
    pub fn new() -> Self {
        Self {
            specs: BTreeMap::new(),
            values: BTreeMap::new(),
            spec_repository: NoopValueSpecRepository,
            data_repository: NoopValueDataRepository,
        }
    }
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";

    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        result.push(HEX[(byte >> 4) as usize] as char);
        result.push(HEX[(byte & 0x0f) as usize] as char);
    }

    result
}

fn decode_hex(hex: &str) -> Result<Vec<u8>, ()> {
    if hex.len() % 2 != 0 {
        return Err(());
    }

    hex.as_bytes()
        .chunks_exact(2)
        .map(|chunk| {
            let high = decode_hex_digit(chunk[0])?;
            let low = decode_hex_digit(chunk[1])?;
            Ok((high << 4) | low)
        })
        .collect()
}

fn decode_hex_digit(byte: u8) -> Result<u8, ()> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(()),
    }
}

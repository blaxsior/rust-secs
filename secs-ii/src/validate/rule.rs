use alloc::string::String;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use crate::item::Secs2FormatCode;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommonRule {
    #[serde(default)]
    pub required: bool,

    #[serde(default)]
    pub repeated: bool,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub length: Option<LengthRule>,

    #[serde(flatten)]
    pub kind: NodeKind,

}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum LengthRule {
    Exact(usize),
    Range(ValueRange),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum NodeKind {
    Scalar(ScalarRule),
    List(ListRule),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScalarRule {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_types: Option<Vec<Secs2FormatCode>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListRule {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<CommonRule>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValueRange {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<i128>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<i128>,
}

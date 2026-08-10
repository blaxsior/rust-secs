use alloc::string::String;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use crate::item::Secs2FormatCode;

/// scalar / list 타입과 무관하게 공통으로 사용되는 규칙
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommonRule {
    /// 필수 여부. 기본 false
    #[serde(default)]
    pub required: bool,

    /// 반복 여부. 기본 false. true인 경우 값이 반복 등장 가능
    #[serde(default)]
    pub repeated: bool,

    /// 아이템의 개수 (length bytes X, 순수한 아이템의 개수)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count: Option<LengthRule>,

    /// 노드의 타입. scalar or list
    #[serde(flatten)]
    pub kind: NodeKind,

}

/// 길이 규칙
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum LengthRule {
    /// 정확한 길이
    Exact(usize),
    Range(ValueRange),
}

/// 노드의 타입. scalar or list
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

/// list에 대한 자식 명시
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListRule {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<CommonRule>,
}

fn val_min_default() -> usize { 0 }
fn val_max_default() -> usize { usize::MAX }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValueRange {
    /// 최소 길이. default = 0
    #[serde(default = "val_min_default")]
    pub min: usize,

    /// 최대 길이. default = max
    #[serde(default = "val_max_default")]
    pub max: usize,
}

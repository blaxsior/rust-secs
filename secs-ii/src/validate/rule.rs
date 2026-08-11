use alloc::string::String;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use crate::item::Secs2FormatCode;

/// scalar / list 타입과 무관하게 공통으로 사용하는 규칙
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommonRule {
    /// 현재 rule이 등장해야 하는 횟수.
    /// root item -> payload 존재 여부 검증
    /// list item -> list 내 child cardinality 검증
    #[serde(default = "ValueRange::single")]
    pub cardinality: ValueRange,

    /// 아이템의 길이 (length bytes X, 순수한 아이템의 길이)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub len: Option<ValueRange>,

    /// 노드 타입 scalar or list
    #[serde(flatten)]
    pub kind: NodeKind,
}

/// 노드 타입 scalar or list
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum NodeKind {
    /// 스칼라 노드
    Scalar(ScalarRule),
    /// 리스트 노드
    List(ListRule),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScalarRule {
    /// 허용되는 scalar 타입
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_types: Option<Vec<Secs2FormatCode>>,

    /// regex 비교 패턴
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
}

/// list에 대한 자식 명시
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListRule {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<CommonRule>,
}

fn val_min_default() -> usize {
    0
}

fn val_max_default() -> usize {
    usize::MAX
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValueRange {
    /// 최소 개수. default = 0
    #[serde(default = "val_min_default")]
    pub min: usize,

    /// 최대 개수. default = max
    #[serde(default = "val_max_default")]
    pub max: usize,
}

impl ValueRange {
    /// 단일 아이템 범위 생성
    pub fn single() -> Self {
        Self { min: 1, max: 1 }
    }

    pub fn optional() -> Self {
        Self { min: 0, max: 1 }
    }
}

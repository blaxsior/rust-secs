#![cfg_attr(not(test), no_std)]

use crate::item::Secs2Variant;

extern crate alloc;

pub mod item;
pub mod convert;
pub mod error;

/// SECS-II Stream Id
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamId(pub u8);

/// SECS-II Function Id
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FunctionId(pub u8);

/// Secs Message 통신 시 사용하는 기본 메시지를 정의한다.
pub struct SecsMessage {
    /// SECS-II stream
    stream: StreamId,
    /// SECS-II function
    function: FunctionId,
    /// 응답이 필요한지 여부
    need_reply: bool,
    /// 메시지 본문
    body: Secs2Variant
}
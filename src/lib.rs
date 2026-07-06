#![cfg_attr(not(test), no_std)]

use crate::item::Secs2Variant;

extern crate alloc;

pub mod convert;
pub mod error;
pub mod item;

/// SECS-II Device Id
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct DeviceId(pub u16);

/// SECS-II Stream Id
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamId(pub u8);

/// SECS-II Function Id
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FunctionId(pub u8);

/// Secs Message 통신 시 사용하는 기본 메시지를 정의한다.
pub struct SecsMessage {
    /// SECS-II stream
    pub stream: StreamId,
    /// SECS-II function
    pub function: FunctionId,
    /// 응답이 필요한지 여부
    pub need_reply: bool,
    /// 메시지 본문
    pub body: Secs2Variant,
}

impl SecsMessage {
    pub fn new(
        stream: StreamId,
        function: FunctionId,
        need_reply: bool,
        body: Secs2Variant,
    ) -> Self {
        Self {
            stream,
            function,
            need_reply,
            body,
        }
    }
}

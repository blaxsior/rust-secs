use alloc::boxed::Box;

use crate::MachineError;
use crate::SecsTimeoutUnit;
use crate::message::RuntimeMessage;
use crate::timer::TimeoutTicket;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachineSignal {
    LinkOpened,
    LinkClosed,
    WriteCompleted,
    WriteFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachineEvent {
    LinkOpenRequested,
    LinkCloseRequested,
}

/// 메시지 전송을 담당하는 구조체. 통신 방식은 내부에 숨기고 있음
pub trait MessageTransport: Send {
    fn start(&mut self) -> Result<(), MachineError>;

    /// 내부 진행을 한 번 수행한다.
    /// datasource read/write, timeout effect 처리 등을 여기서 drive.
    fn poll(&mut self) -> Result<(), MachineError>;

    /// outbound message를 큐에 담는다.
    fn send(&mut self, msg: RuntimeMessage) -> Result<(), MachineError>;

    /// 이미 수신 완료된 메시지를 꺼낸다.
    fn poll_recv(&mut self) -> Option<RuntimeMessage>;

    fn handle_timeout(&mut self, ticket: TimeoutTicket) -> Result<(), MachineError>;

    fn poll_timeout(&mut self) -> Option<TimeoutTicket>;

    fn poll_expired_timeout(&mut self) -> Option<SecsTimeoutUnit> {
        None
    }
}

impl<T> MessageTransport for Box<T>
where
    T: MessageTransport + ?Sized,
{
    fn start(&mut self) -> Result<(), MachineError> {
        self.as_mut().start()
    }

    fn poll(&mut self) -> Result<(), MachineError> {
        self.as_mut().poll()
    }

    fn send(&mut self, msg: RuntimeMessage) -> Result<(), MachineError> {
        self.as_mut().send(msg)
    }

    fn poll_recv(&mut self) -> Option<RuntimeMessage> {
        self.as_mut().poll_recv()
    }

    fn handle_timeout(&mut self, ticket: TimeoutTicket) -> Result<(), MachineError> {
        self.as_mut().handle_timeout(ticket)
    }

    fn poll_timeout(&mut self) -> Option<TimeoutTicket> {
        self.as_mut().poll_timeout()
    }

    fn poll_expired_timeout(&mut self) -> Option<SecsTimeoutUnit> {
        self.as_mut().poll_expired_timeout()
    }
}

use alloc::collections::VecDeque;

use crate::transport::ConnectionRole::{Active, Passive};
use crate::transport::error::SecsTransportError;
use crate::transport::hsms::connection::HsmsConnectionState::NotSelected;
use crate::transport::hsms::{HsmsHeader, HsmsSType};
use crate::transport::{ConnectionRole, SecsTimeoutUnit};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HsmsConnectionState {
    NotConnected,
    NotSelected,
    Selected,
}

impl HsmsConnectionState {
    pub fn is_connected(&self) -> bool {
        !matches!(self, Self::NotConnected)
    }

    pub fn is_selected(&self) -> bool {
        matches!(self, Self::Selected)
    }

    pub fn is_not_selected(&self) -> bool {
        matches!(self, Self::NotSelected)
    }
}

/// 외부에서 전달된 신호
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HsmsConnSignal {
    HsmsControl(HsmsHeader),
    TcpConnected,
    TcpDisconnected,
    Timeout(SecsTimeoutUnit),
}

/// 외부로 현재 액션을 알림
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HsmsConnEvent {
    Connected,
    Selected,
    Deselected,
    Separated,
    ConnectionFailed,
}

/// 외부로 전달하는 요청
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HsmsConnEffect {
    OpenSession,
    CloseSession,
    SendControl(HsmsSType),
    StartTimeout(SecsTimeoutUnit),
    ClearTimeout(SecsTimeoutUnit),
    Notify(HsmsConnEvent),
}

/// HSMS session manager.
///
pub struct HsmsSessionManager {
    state: HsmsConnectionState,
    role: ConnectionRole,
    can_reconnect: bool,
    /// 외부로 요구하는 요청
    effects: VecDeque<HsmsConnEffect>,
}

impl HsmsSessionManager {
    pub fn new(role: ConnectionRole) -> Self {
        Self {
            state: HsmsConnectionState::NotConnected,
            role,
            can_reconnect: true,
            effects: VecDeque::new(),
        }
    }

    /// p
    pub fn handle_active_session(&mut self) {}

    pub fn handle_passive_session(&mut self) {}

    /// 현재 상태
    pub fn state(&self) -> HsmsConnectionState {
        self.state
    }
    pub fn handle(&mut self, signal: HsmsConnSignal) {
        match signal {
            HsmsConnSignal::HsmsControl(hsms_header) => todo!(),
            HsmsConnSignal::TcpConnected => todo!(),
            HsmsConnSignal::TcpDisconnected => todo!(),
            HsmsConnSignal::Timeout(secs_timeout_unit) => todo!(),
        }
       
    }

    /// 특정 요청 처리 가능한 상태인지 여부 반환
    /// select.req -> not selected && active
    /// link / data / separate -> selected
    /// reject -> always (조건에 따라 사용됨)
    /// deselect -> 사용되지 않음
    pub fn is_allowed(&self, header: &HsmsHeader) -> bool {
        match header.stype {
            HsmsSType::DataMessage => self.state.is_selected(),
            HsmsSType::SelectRequest => self.state.is_not_selected() && self.role == Active,
            HsmsSType::SelectResponse => self.state.is_not_selected() && self.role == Passive,
            HsmsSType::DeselectRequest => false,
            HsmsSType::DeselectResponse => false,
            HsmsSType::LinktestRequest => self.state.is_selected(),
            HsmsSType::LinktestResponse => self.state.is_selected(),
            HsmsSType::RejectRequest => true,
            HsmsSType::SeparateRequest => self.state.is_selected(),
        }
    }

    pub fn can_connect(&self) -> bool {
        self.can_reconnect
    }

    /// tcp 연결 시 호출
    pub fn on_tcp_connect(&mut self) -> Result<(), SecsTransportError> {
        if matches!(self.state, HsmsConnectionState::NotConnected) {
            self.change_state(HsmsConnectionState::NotSelected);
            Ok(())
        } else {
            // 이전 상태가 Not Connected인 경우에만 NotSelected로 전환 가능
            // TODO: invalid state 내용 구체화
            Err(SecsTransportError::InvalidState)
        }
    }

    fn change_state(&mut self, state: HsmsConnectionState) {
        self.state = state;
    }
}

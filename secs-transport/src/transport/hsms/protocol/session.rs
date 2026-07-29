use alloc::vec::Vec;
use secs_common::SessionId;

use crate::transport::ConnectionRole::{Active, Passive};
use crate::transport::hsms::{HsmsControl, HsmsHeader, HsmsSType, HsmsSelectStatus};
use crate::transport::{ConnectionRole, SecsTimeoutUnit};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HsmsConnectionState {
    NotConnected,
    NotSelected,
    Selected,
}

impl HsmsConnectionState {
    /// 연결 되어 있는지 여부를 반환. Selected / NotSelected가 Connected의 하위 타입
    pub fn is_connected(&self) -> bool {
        !matches!(self, Self::NotConnected)
    }

    /// not_connected 상태인지 체크
    pub fn is_not_connected(&self) -> bool {
        matches!(self, Self::NotConnected)
    }

    /// selected 상태인지
    pub fn is_selected(&self) -> bool {
        matches!(self, Self::Selected)
    }

    /// not selected 상태인지
    pub fn is_not_selected(&self) -> bool {
        matches!(self, Self::NotSelected)
    }
}

/// 외부에서 전달된 신호
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HsmsSessionSignal {
    /// Control 메시지 수신
    RecvControl(HsmsControl),
    /// 연결됨
    Connected,
    /// 연결 끊김
    Disconnected,
    /// timeout 발생
    Timeout(SecsTimeoutUnit),
}

/// 외부로 전달하는 요청
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HsmsSessionEffect {
    Connect,
    Disconnect,
    SendControl(HsmsControl),
    StartTimeout(SecsTimeoutUnit),
    ClearTimeout(SecsTimeoutUnit),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HsmsSessionError {
    InvalidState(HsmsConnectionState),
    ConnectNotAllowed,
}

pub type HsmsSessionResult<T> = Result<T, HsmsSessionError>;

pub struct HsmsSession {
    state: HsmsConnectionState,
    session_id: SessionId,
    /// timeout 발생 후 세션을 다시 시작할 것인지 여부. seperate로 종료 시 재시작 X
    can_connect: bool,
    role: ConnectionRole,

    /// 외부로 보내는 신호
    effects: Vec<HsmsSessionEffect>,
}

impl HsmsSession {
    pub fn new(session_id: SessionId, role: ConnectionRole) -> Self {
        Self {
            state: HsmsConnectionState::NotConnected,
            session_id,
            can_connect: true,
            role,
            effects: Vec::new(),
        }
    }

    /// TCP connect 요청 전송
    pub fn is_passive(&self) -> bool {
        self.role.is_passive()
    }

    pub fn is_active(&self) -> bool {
        self.role.is_active()
    }

    pub fn connect(&mut self) -> HsmsSessionResult<Vec<HsmsSessionEffect>> {
        if self.state.is_connected() {
            log::error!("already connected");
            return self.fail(HsmsSessionError::InvalidState(self.state));
        }

        if !self.can_connect {
            log::error!("cannot connect because of T5");
            return self.fail(HsmsSessionError::ConnectNotAllowed);
        }

        self.stack_effect(HsmsSessionEffect::Connect);
        Ok(self.emit())
    }

    /// TCP 연결 해제 요청 전송
    pub fn disconnect(&mut self) -> HsmsSessionResult<Vec<HsmsSessionEffect>> {
        self.stack_effect(HsmsSessionEffect::Disconnect);
        Ok(self.emit())
    }

    /// select request
    pub fn select(&mut self) -> HsmsSessionResult<Vec<HsmsSessionEffect>> {
        if !self.state.is_not_selected() || self.role.is_passive() {
            log::error!("already selected");
            return self.fail(HsmsSessionError::InvalidState(self.state));
        }

        self.send_control(HsmsControl::SelectReq);
        self.start_timeout(SecsTimeoutUnit::T6);

        Ok(self.emit())
    }

    /// linktest 요청
    pub fn linktest(&mut self) -> HsmsSessionResult<Vec<HsmsSessionEffect>> {
        if !self.state.is_selected() {
            log::error!("linktest not allowed in not selected state");
            return self.fail(HsmsSessionError::InvalidState(self.state));
        }

        self.send_control(HsmsControl::LinktestReq);
        self.start_timeout(SecsTimeoutUnit::T6);

        Ok(self.emit())
    }

    /// seperate 요청. separate 요청 보내는 동시에 연결 끊음
    pub fn separate(&mut self) -> HsmsSessionResult<Vec<HsmsSessionEffect>> {
        if !self.state.is_selected() {
            log::error!("seperate not allowed in not selected state");
            return self.fail(HsmsSessionError::InvalidState(self.state));
        }

        self.send_control(HsmsControl::SeparateReq);
        self.stack_effect(HsmsSessionEffect::Disconnect);

        Ok(self.emit())
    }

    /// 현재 상태
    pub fn state(&self) -> HsmsConnectionState {
        self.state
    }

    /// control 요청 한다.
    fn send_control(&mut self, control: HsmsControl) {
        self.stack_effect(HsmsSessionEffect::SendControl(control));
    }

    fn start_timeout(&mut self, timeout: SecsTimeoutUnit) {
        self.stack_effect(HsmsSessionEffect::StartTimeout(timeout));
    }

    fn cancel_timeout(&mut self, timeout: SecsTimeoutUnit) {
        self.stack_effect(HsmsSessionEffect::ClearTimeout(timeout));
    }

    /// 외부 신호를 처리. req / res 모두 상태 체크 대상
    pub fn handle(
        &mut self,
        signal: HsmsSessionSignal,
    ) -> HsmsSessionResult<Vec<HsmsSessionEffect>> {
        // 현재 상태를 기준으로 신호에 대응
        match self.state {
            HsmsConnectionState::NotConnected => self.handle_not_connected(signal),
            HsmsConnectionState::NotSelected => self.handle_not_selected(signal),
            HsmsConnectionState::Selected => self.handle_selected(signal),
        }
    }

    /// 대상 메시지 처리 가능 여부
    /// select -> not selected
    /// link / data / separate -> selected
    /// reject -> always (기본 optional)
    /// deselect -> not used
    pub fn is_allowed(&self, header: &HsmsHeader) -> bool {
        match header.stype {
            HsmsSType::DataMessage => self.state.is_selected(),
            // SELECT는 ACTIVE 주도, PASSIVE 응답
            HsmsSType::SelectReq => self.state.is_not_selected(),
            HsmsSType::SelectRsp => self.state.is_not_selected(),
            // DESELECT는 E37.1 에서 미사용
            HsmsSType::DeselectReq => false,
            HsmsSType::DeselectRsp => false,
            HsmsSType::LinktestReq => self.state.is_selected(),
            HsmsSType::LinktestRsp => self.state.is_selected(),
            HsmsSType::RejectReq => self.state.is_connected(),
            HsmsSType::SeparateReq => self.state.is_selected(),
        }
    }

    /// not connected 상태일 때의 상태 전이 대응
    fn handle_not_connected(
        &mut self,
        signal: HsmsSessionSignal,
    ) -> HsmsSessionResult<Vec<HsmsSessionEffect>> {
        match signal {
            HsmsSessionSignal::Connected => {
                if !self.state.is_connected() {
                    self.change_state(HsmsConnectionState::NotSelected);

                    match self.role {
                        Active => {
                            // active인 경우 Select 요청 보내기 + T6 timeout 시작
                            return self.select();
                        }
                        Passive => {
                            // passive인 경우 T7 timeout을 시작
                            self.start_timeout(SecsTimeoutUnit::T7);
                        }
                    }
                } else {
                    log::warn!("tcp already connected");
                }
            }
            // not connected일 때 다시 해당 신호를 받은 경우
            HsmsSessionSignal::Disconnected => {
                log::warn!("tcp already disconnected");
            }
            HsmsSessionSignal::Timeout(unit) => {
                if matches!(unit, SecsTimeoutUnit::T5) && self.role.is_active() {
                    // T5 timeout이 발생, 내가 active 인 경우 reconnect 시도
                    self.can_connect = true; // 재연결 가능함 알림
                    self.stack_effect(HsmsSessionEffect::Connect);
                }
            }
            HsmsSessionSignal::RecvControl(..) => {
                log::error!("control when not connected... ignore. {:?}", signal);
            }
        }

        Ok(self.emit())
    }

    fn handle_not_selected(
        &mut self,
        signal: HsmsSessionSignal,
    ) -> HsmsSessionResult<Vec<HsmsSessionEffect>> {
        match signal {
            HsmsSessionSignal::RecvControl(control) => {
                match self.role {
                    Active => {
                        // select success 인 케이스
                        if matches!(control, HsmsControl::SelectRsp(status) if status == HsmsSelectStatus::Success)
                        {
                            self.cancel_timeout(SecsTimeoutUnit::T6);
                            self.change_state(HsmsConnectionState::Selected);
                        } else {
                            // 연결 문제 있으므로 연결 해제
                            return self.disconnect();
                        }
                    }
                    Passive => {
                        if matches!(control, HsmsControl::SelectReq) {
                            // select response 반환
                            self.cancel_timeout(SecsTimeoutUnit::T7);
                            // TODO: 외부 신호 받아서 selected 가능 여부 조사 후 메시지 전송
                            // 현재는 항상 SUCCESS로 간주 중
                            self.send_control(HsmsControl::SelectRsp(HsmsSelectStatus::Success));
                            // select로 상태 전이
                            self.change_state(HsmsConnectionState::Selected);
                        } else {
                            // select.req 이외 수신 시 연결 종료
                            log::error!(
                                "control not allowed, disconnect connection. state = {:?}, cont = {:?}",
                                self.state,
                                control
                            );
                            return self.disconnect();
                        }
                    }
                }
            }
            HsmsSessionSignal::Connected => {
                log::warn!("tcp already connected");
            }
            HsmsSessionSignal::Disconnected => {
                log::debug!("tcp disconnected. return to not connected state");
                self.change_state(HsmsConnectionState::NotConnected);
                self.passive_reconnect();
                if self.role.is_active() {
                    // not selected -> not selected 시 T5 timeout 시작
                    self.can_connect = false;
                    self.stack_effect(HsmsSessionEffect::StartTimeout(SecsTimeoutUnit::T5));
                }
            }
            HsmsSessionSignal::Timeout(unit) => {
                match self.role {
                    Active => {
                        // control 중 T6 발생 or TCP 통신 중 T8 발생 -> TCP 커넥션 종료
                        if matches!(unit, SecsTimeoutUnit::T6 | SecsTimeoutUnit::T8) {
                            self.stack_effect(HsmsSessionEffect::Disconnect);
                        }
                    }
                    Passive => {
                        // select.req 대기 중 T7 발생 or TCP 통신 중 T8 발생 -> TCP 커넥션 종료
                        if matches!(unit, SecsTimeoutUnit::T7 | SecsTimeoutUnit::T8) {
                            self.stack_effect(HsmsSessionEffect::Disconnect);
                        }
                    }
                }
            }
        }
        Ok(self.emit())
    }

    fn handle_selected(
        &mut self,
        signal: HsmsSessionSignal,
    ) -> HsmsSessionResult<Vec<HsmsSessionEffect>> {
        match signal {
            HsmsSessionSignal::RecvControl(control) => {
                match self.role {
                    Active => {
                        match control {
                            HsmsControl::SelectRsp(..) => {
                                log::warn!("already selected but recv select.rsp {:?}", control);
                            }
                            HsmsControl::DeselectReq | HsmsControl::DeselectRsp(..) => {
                                log::warn!("control {:?} not used in E37.1", control);
                            }
                            // 상대방이 linktest 요청
                            HsmsControl::LinktestReq => {
                                self.send_control(HsmsControl::LinktestRsp);
                            }
                            HsmsControl::LinktestRsp => {
                                // linktest에 대한 T6 timeout 초기화
                                self.cancel_timeout(SecsTimeoutUnit::T6);
                                log::info!("linktest success");
                            }
                            HsmsControl::SeparateReq => {
                                // seperate 요청 받음 -> disconnect 요청
                                log::debug!("separate requested. disconnect");
                                self.stack_effect(HsmsSessionEffect::Disconnect);
                            }
                            HsmsControl::RejectReq(..) => {}
                            _ => {
                                // 이상한 데이터를 수신한 경우 메시지 거절
                                log::error!("recv wrong {:?}", control);
                            }
                        }
                    }
                    Passive => {
                        match control {
                            HsmsControl::SelectReq => {
                                log::warn!("already selected but recv select.req");
                                // 이미 select 상태임을 알림
                                self.send_control(HsmsControl::SelectRsp(
                                    HsmsSelectStatus::AlreadyActive,
                                ));
                            }
                            HsmsControl::DeselectReq | HsmsControl::DeselectRsp(..) => {
                                log::warn!("control {:?} not used in E37.1", control);
                            }
                            // 상대방이 linktest 요청
                            HsmsControl::LinktestReq => {
                                self.send_control(HsmsControl::LinktestRsp);
                            }
                            HsmsControl::LinktestRsp => {
                                // linktest에 대한 T6 timeout 초기화
                                self.cancel_timeout(SecsTimeoutUnit::T6);
                                log::info!("linktest success");
                            }
                            HsmsControl::SeparateReq => {
                                // seperate 요청 받음 -> disconnect 요청
                                log::debug!("separate requested. disconnect");
                                self.stack_effect(HsmsSessionEffect::Disconnect);
                            }
                            HsmsControl::RejectReq(..) => {}
                            _ => {
                                // 이상한 데이터를 수신한 경우 메시지 거절
                                log::error!("recv wrong {:?}", control);
                            }
                        }
                    }
                }
            }
            HsmsSessionSignal::Connected => {
                log::warn!("tcp already connected");
            }
            HsmsSessionSignal::Disconnected => {
                self.change_state(HsmsConnectionState::NotConnected);
                self.passive_reconnect();
            }
            HsmsSessionSignal::Timeout(unit) => {
                // control 중 T6 발생 or TCP 통신 중 T8 발생 -> TCP 커넥션 종료
                if matches!(unit, SecsTimeoutUnit::T6 | SecsTimeoutUnit::T8) {
                    self.stack_effect(HsmsSessionEffect::Disconnect);
                }
            }
        }

        Ok(self.emit())
    }

    fn change_state(&mut self, state: HsmsConnectionState) {
        log::debug!("state: {:?} → {:?}", self.state, state);
        self.state = state;
    }

    /// effect를 호출
    fn stack_effect(&mut self, effect: HsmsSessionEffect) {
        self.effects.push(effect);
    }

    fn emit(&mut self) -> Vec<HsmsSessionEffect> {
        core::mem::take(&mut self.effects)
    }

    /// 작업 도중 실패 처리
    fn fail<T>(&mut self, error: HsmsSessionError) -> HsmsSessionResult<T> {
        // 실패한 작업에 대한 effect 정리
        self.effects.clear();
        Err(error)
    }

    /// passive 상태에서 connect 대기 상태로 돌아가기 위해 필요한 함수
    /// datasource server 구현 방식이 현재와 달라질 경우 필요하지 않을 수 있음
    /// 세션을 자동 연결할 수 있도록 기능 추가
    fn passive_reconnect(&mut self) {
        if self.role.is_passive() {
            self.stack_effect(HsmsSessionEffect::Connect);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_active_session() -> HsmsSession {
        HsmsSession::new(SessionId(1), ConnectionRole::Active)
    }

    fn get_passive_session() -> HsmsSession {
        HsmsSession::new(SessionId(1), ConnectionRole::Passive)
    }

    /// active connect 성공 시 상태 전이. Select.req 전송 + T6 timeout 시작
    #[test]
    fn test_active_after_connect() {
        let mut session = get_active_session();
        let effects = session.handle(HsmsSessionSignal::Connected).unwrap();

        assert_eq!(session.state, HsmsConnectionState::NotSelected);
        assert!(effects.contains(&HsmsSessionEffect::StartTimeout(SecsTimeoutUnit::T6)));
        assert!(effects.contains(&HsmsSessionEffect::SendControl(HsmsControl::SelectReq)));
    }

    /// active not_selected 상태에서 disconnect 시 t5 timeout 발생
    #[test]
    fn test_active_not_selected_if_disconnect_start_t5_timeout() {
        let mut session = get_active_session();
        session.state = HsmsConnectionState::NotSelected;

        // 연결 끊겼음
        let effects = session.handle(HsmsSessionSignal::Disconnected).unwrap();

        assert_eq!(session.state, HsmsConnectionState::NotConnected);
        // 해당 시간동안은 reconnect 불가
        assert_eq!(session.can_connect, false);
        assert!(effects.contains(&HsmsSessionEffect::StartTimeout(SecsTimeoutUnit::T5)));
    }

    /// active not_selected 상태에서 T6 timeout 발생 시 disconnect
    #[test]
    fn test_active_not_selected_t6_timout_occured() {
        let mut session = get_active_session();
        session.state = HsmsConnectionState::NotSelected;

        // timeout 발생
        let effects = session
            .handle(HsmsSessionSignal::Timeout(SecsTimeoutUnit::T6))
            .unwrap();

        assert_eq!(session.state, HsmsConnectionState::NotSelected);
        assert!(effects.contains(&HsmsSessionEffect::Disconnect));
    }

    /// active not_selected 상태에서 select.rsp 외 응답 받을 시 disconnect
    #[test]
    fn test_active_not_selected_recv_non_select_rsp() {
        let mut session = get_active_session();
        session.state = HsmsConnectionState::NotSelected;

        // non_select 받음
        let effects = session
            .handle(HsmsSessionSignal::RecvControl(HsmsControl::LinktestReq))
            .unwrap();

        assert_eq!(session.state, HsmsConnectionState::NotSelected);
        assert!(effects.contains(&HsmsSessionEffect::Disconnect));
    }

    /// active not_selected 상태에서 select.rsp 실패 메시지를 받은 경우 disconnect
    #[test]
    fn test_active_not_selected_recv_select_rsp_with_err() {
        let mut session = get_active_session();
        session.state = HsmsConnectionState::NotSelected;

        // non_select 받음
        let effects = session
            .handle(HsmsSessionSignal::RecvControl(HsmsControl::SelectRsp(
                HsmsSelectStatus::NotReady,
            )))
            .unwrap();

        assert_eq!(session.state, HsmsConnectionState::NotSelected);
        assert!(effects.contains(&HsmsSessionEffect::Disconnect));
    }

    /// active not_selected 상태에서 select.rsp 성공 시 timeout clear, 상태 전이
    #[test]
    fn test_active_not_selected_recv_select_rsp_ok_then_move_to_selected() {
        let mut session = get_active_session();
        session.state = HsmsConnectionState::NotSelected;

        // non_select 받음
        let effects = session
            .handle(HsmsSessionSignal::RecvControl(HsmsControl::SelectRsp(
                HsmsSelectStatus::Success,
            )))
            .unwrap();

        assert_eq!(session.state, HsmsConnectionState::Selected);
        assert!(effects.contains(&HsmsSessionEffect::ClearTimeout(SecsTimeoutUnit::T6)));
    }

    /// connect 성공 시 상태 전이. Select.req 전송 + T6 timeout 시작
    #[test]
    fn test_passive_after_connect() {
        let mut session = get_passive_session();
        let effects = session.handle(HsmsSessionSignal::Connected).unwrap();

        assert_eq!(session.state, HsmsConnectionState::NotSelected);
        assert!(effects.contains(&HsmsSessionEffect::StartTimeout(SecsTimeoutUnit::T7)));
    }

    /// passive not_selected 상태에서 select.req 외 요청 받을 시 disconnect
    #[test]
    fn test_passive_not_selected_recv_non_select_req() {
        let mut session = get_passive_session();
        session.state = HsmsConnectionState::NotSelected;

        // not select.req
        let effects = session
            .handle(HsmsSessionSignal::RecvControl(HsmsControl::LinktestReq))
            .unwrap();

        assert_eq!(session.state, HsmsConnectionState::NotSelected);
        assert!(effects.contains(&HsmsSessionEffect::Disconnect));
    }

    /// passive not selected 상태에서 T7 timeout 발생 시 disconnect 요청
    #[test]
    fn test_passive_not_selected_t7_timeout_request_disconnect() {
        let mut session = get_passive_session();
        session.state = HsmsConnectionState::NotSelected;

        let effects = session
            .handle(HsmsSessionSignal::Timeout(SecsTimeoutUnit::T7))
            .unwrap();

        assert!(effects.contains(&HsmsSessionEffect::Disconnect));
    }

    /// passive not_selected 상태에서 select.req 메시지를 받은 경우 T7 timeout clear, rsp 보내고 select 전이
    #[test]
    fn test_passive_not_selected_recv_select_req_move_to_selected() {
        let mut session = get_passive_session();
        session.state = HsmsConnectionState::NotSelected;

        // select req 받음
        let effects = session
            .handle(HsmsSessionSignal::RecvControl(HsmsControl::SelectReq))
            .unwrap();

        assert_eq!(session.state, HsmsConnectionState::Selected);

        assert!(effects.contains(&HsmsSessionEffect::ClearTimeout(SecsTimeoutUnit::T7)));
        assert!(effects.iter().any(|it| matches!(
            it,
            &HsmsSessionEffect::SendControl(HsmsControl::SelectRsp(HsmsSelectStatus::Success))
        )));
    }

    /// T5 대기 중 connect 요청 시 에러 반환
    #[test]
    fn test_passive_not_selected_disconnect_reconnects() {
        let mut session = get_passive_session();
        session.state = HsmsConnectionState::NotSelected;

        let effects = session.handle(HsmsSessionSignal::Disconnected).unwrap();

        assert_eq!(session.state, HsmsConnectionState::NotConnected);
        assert!(effects.contains(&HsmsSessionEffect::Connect));
    }

    #[test]
    fn test_passive_selected_disconnect_reconnects() {
        let mut session = get_passive_session();
        session.state = HsmsConnectionState::Selected;

        let effects = session.handle(HsmsSessionSignal::Disconnected).unwrap();

        assert_eq!(session.state, HsmsConnectionState::NotConnected);
        assert!(effects.contains(&HsmsSessionEffect::Connect));
    }

    #[test]
    fn test_connect_not_allowed_if_t5_waiting() {
        let mut session = get_active_session();
        session.can_connect = false;

        let result = session.connect();

        assert_eq!(result, Err(HsmsSessionError::ConnectNotAllowed));
    }

    /// connect는 not connected + can_connect 상태에서 허용
    #[test]
    fn test_connect_allowed_if_not_connected() {
        let mut session = get_active_session();

        let effects = session.connect().unwrap();

        assert_eq!(session.state, HsmsConnectionState::NotConnected);
        assert_eq!(effects, vec![HsmsSessionEffect::Connect]);
    }

    /// connect는 not connected 상태에서만 허용
    #[test]
    fn test_connect_invalid_if_already_connected() {
        let mut session = get_active_session();
        session.state = HsmsConnectionState::NotSelected;

        let result = session.connect();

        assert_eq!(
            result,
            Err(HsmsSessionError::InvalidState(
                HsmsConnectionState::NotSelected
            ))
        );
    }

    /// connect는 selected 상태에서 허용하지 않음
    #[test]
    fn test_connect_invalid_if_selected() {
        let mut session = get_active_session();
        session.state = HsmsConnectionState::Selected;

        let result = session.connect();

        assert_eq!(
            result,
            Err(HsmsSessionError::InvalidState(
                HsmsConnectionState::Selected
            ))
        );
    }

    /// 에러 발생 시 기존에 쌓인 effect를 제거
    #[test]
    fn test_error_clears_pending_effects() {
        let mut session = get_active_session();
        session.stack_effect(HsmsSessionEffect::Connect);
        session.state = HsmsConnectionState::Selected;

        let result = session.connect();

        assert_eq!(
            result,
            Err(HsmsSessionError::InvalidState(
                HsmsConnectionState::Selected
            ))
        );
        assert!(session.effects.is_empty());
    }

    /// select는 active + not selected 상태에서 허용
    #[test]
    fn test_select_allowed_if_active_not_selected() {
        let mut session = get_active_session();
        session.state = HsmsConnectionState::NotSelected;

        let effects = session.select().unwrap();

        assert!(effects.contains(&HsmsSessionEffect::SendControl(HsmsControl::SelectReq)));
        assert!(effects.contains(&HsmsSessionEffect::StartTimeout(SecsTimeoutUnit::T6)));
    }

    /// select는 not selected 상태에서만 허용
    #[test]
    fn test_select_invalid_if_not_connected() {
        let mut session = get_active_session();

        let result = session.select();

        assert_eq!(
            result,
            Err(HsmsSessionError::InvalidState(
                HsmsConnectionState::NotConnected
            ))
        );
    }

    /// select는 passive에서 허용하지 않음
    #[test]
    fn test_select_invalid_if_passive() {
        let mut session = get_passive_session();
        session.state = HsmsConnectionState::NotSelected;

        let result = session.select();

        assert_eq!(
            result,
            Err(HsmsSessionError::InvalidState(
                HsmsConnectionState::NotSelected
            ))
        );
    }

    /// select는 selected 상태에서 허용하지 않음
    #[test]
    fn test_select_invalid_if_selected() {
        let mut session = get_active_session();
        session.state = HsmsConnectionState::Selected;

        let result = session.select();

        assert_eq!(
            result,
            Err(HsmsSessionError::InvalidState(
                HsmsConnectionState::Selected
            ))
        );
    }

    /// linktest는 selected 상태에서 허용
    #[test]
    fn test_linktest_allowed_if_selected() {
        let mut session = get_active_session();
        session.state = HsmsConnectionState::Selected;

        let effects = session.linktest().unwrap();

        assert!(effects.contains(&HsmsSessionEffect::SendControl(HsmsControl::LinktestReq)));
        assert!(effects.contains(&HsmsSessionEffect::StartTimeout(SecsTimeoutUnit::T6)));
    }

    /// linktest는 selected 상태에서만 허용
    #[test]
    fn test_linktest_invalid_if_not_selected() {
        let mut session = get_active_session();
        session.state = HsmsConnectionState::NotSelected;

        let result = session.linktest();

        assert_eq!(
            result,
            Err(HsmsSessionError::InvalidState(
                HsmsConnectionState::NotSelected
            ))
        );
    }

    /// separate는 selected 상태에서 허용
    #[test]
    fn test_separate_allowed_if_selected() {
        let mut session = get_active_session();
        session.state = HsmsConnectionState::Selected;

        let effects = session.separate().unwrap();

        assert!(effects.contains(&HsmsSessionEffect::SendControl(HsmsControl::SeparateReq)));
        assert!(effects.contains(&HsmsSessionEffect::Disconnect));
    }

    /// separate는 selected 상태에서만 허용
    #[test]
    fn test_separate_invalid_if_not_selected() {
        let mut session = get_active_session();
        session.state = HsmsConnectionState::NotSelected;

        let result = session.separate();

        assert_eq!(
            result,
            Err(HsmsSessionError::InvalidState(
                HsmsConnectionState::NotSelected
            ))
        );
    }
}

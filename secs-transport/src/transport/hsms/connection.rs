use alloc::collections::VecDeque;
use alloc::vec::Vec;

use crate::transport::ConnectionRole::{Active, Passive};
use crate::transport::error::SecsTransportError;
use crate::transport::hsms::{HsmsHeader, HsmsSType, HsmsSelectStatus};
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

    pub fn is_selected(&self) -> bool {
        matches!(self, Self::Selected)
    }

    pub fn is_not_selected(&self) -> bool {
        matches!(self, Self::NotSelected)
    }
}

// /// 외부에서 전달된 신호
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum HsmsConnSignal {
//     /// Control 메시지 수신
//     RecvControl(HsmsHeader),
//     /// TCP 연결됨
//     TcpConnected,
//     /// TCP 연결 끊김
//     TcpDisconnected,
//     /// timeout 발생
//     Timeout(SecsTimeoutUnit),
// }

// /// 외부로 전달하는 요청
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum HsmsConnEffect {
//     Connect,
//     Disconnect,
//     SendControl(HsmsHeader),
//     StartTimeout(SecsTimeoutUnit),
//     ClearTimeout(SecsTimeoutUnit),
// }

// /// HSMS session manager.
// ///
// pub struct HsmsSessionManager {
//     state: HsmsConnectionState,
//     role: ConnectionRole,
//     can_reconnect: bool,
//     /// 외부로 요구하는 요청
//     effects: VecDeque<HsmsConnEffect>,
// }

// impl HsmsSessionManager {
//     pub fn new(role: ConnectionRole) -> Self {
//         Self {
//             state: HsmsConnectionState::NotConnected,
//             role,
//             can_reconnect: true,
//             effects: VecDeque::new(),
//         }
//     }

//     /// connect 요청 수동 시도
//     pub fn connect(&mut self) {}

//     pub fn select(&mut self) -> Result<(), SecsTransportError> {
//         if
//         // select response 반환
//         let select_rsp = HsmsHeader::control(
//             0,
//             HsmsSelectStatus::Success as u8,
//             HsmsSType::SelectRsp,
//             header.system_byte,
//         );

//         self.emit(effect);
//         self.emit(HsmsConnEffect::StartTimeout(SecsTimeoutUnit::T6));
//     }

//     /// linktest 요청
//     pub fn linktest(&mut self) {}
//     /// deselect 요청
//     // pub fn deselect(&mut self) {}

//     /// seperate 요청
//     pub fn separate(&mut self) {}

//     /// 현재 상태
//     pub fn state(&self) -> HsmsConnectionState {
//         self.state
//     }

//     /// 외부 신호를 처리. req / res 모두 상태 체크 대상
//     pub fn handle(&mut self, signal: HsmsConnSignal) -> Result<(), SecsTransportError> {
//         // 현재 상태를 기준으로 신호에 대응
//         match self.state {
//             HsmsConnectionState::NotConnected => self.handle_not_connected(signal),
//             HsmsConnectionState::NotSelected => self.handle_not_selected(signal),
//             HsmsConnectionState::Selected => self.handle_selected(signal),
//         }
//     }

//     /// 대상 메시지 처리 가능 여부 검사
//     /// select -> not selected
//     /// link / data / separate -> selected
//     /// reject -> always (기본 optional)
//     /// deselect -> not used
//     pub fn is_allowed(&self, header: &HsmsHeader) -> bool {
//         match header.stype {
//             HsmsSType::DataMessage => self.state.is_selected(),
//             // SELECT는 ACTIVE 주도, PASSIVE 응답
//             HsmsSType::SelectReq => self.state.is_not_selected() && self.role.is_active(),
//             HsmsSType::SelectRsp => self.state.is_not_selected() && self.role.is_passive(),
//             // DESELECT는 E37.1 에서 미사용
//             HsmsSType::DeselectReq => false,
//             HsmsSType::DeselectRsp => false,
//             HsmsSType::LinktestReq => self.state.is_selected(),
//             HsmsSType::LinktestRsp => self.state.is_selected(),
//             HsmsSType::RejectReq => true,
//             HsmsSType::SeparateReq => self.state.is_selected(),
//         }
//     }

//     pub fn can_connect(&self) -> bool {
//         self.can_reconnect
//     }

//     /// not connected 상태일 때의 상태 전이 대응
//     fn handle_not_connected(&mut self, signal: HsmsConnSignal) -> Result<(), SecsTransportError> {
//         match signal {
//             HsmsConnSignal::TcpConnected => {
//                 if !self.state.is_connected() {
//                     self.change_state(HsmsConnectionState::NotSelected);

//                     match self.role {
//                         Active => {
//                             // active인 경우 Select 요청 보내기 + T6 timeout 시작
//                             self.emit(effect);
//                         }
//                         Passive => {
//                             // passive인 경우 T7 timeout을 시작
//                             self.emit(HsmsConnEffect::StartTimeout(SecsTimeoutUnit::T7));
//                         }
//                     }
//                 } else {
//                     log::warn!("tcp already connected");
//                 }
//             }
//             // not connected일 때 다시 해당 신호를 받은 경우
//             HsmsConnSignal::TcpDisconnected => {
//                 log::warn!("tcp already disconnected");
//             }
//             HsmsConnSignal::Timeout(unit) => {
//                 if matches!(unit, SecsTimeoutUnit::T5) && self.role.is_active() {
//                     // T5 timeout이 발생, 내가 active 인 경우 reconnect 시도
//                     self.emit(HsmsConnEffect::Connect);
//                 }
//             }
//             HsmsConnSignal::RecvControl(..) => {
//                 log::error!("control when not connected... ignore. {:?}", signal);
//             }
//         }

//         Ok(())
//     }

//     fn handle_not_selected(&mut self, signal: HsmsConnSignal) -> Result<(), SecsTransportError> {
//         match signal {
//             HsmsConnSignal::RecvControl(header) => {
//                 match self.role {
//                     Active => {}
//                     Passive => {
//                         if matches!(header.stype, HsmsSType::SelectReq) {
//                             // select response 반환
//                             let select_rsp = HsmsHeader::control(
//                                 0,
//                                 HsmsSelectStatus::Success as u8,
//                                 HsmsSType::SelectRsp,
//                                 header.system_byte,
//                             );

//                             self.emit(HsmsConnEffect::SendControl(select_rsp));
//                             self.change_state(HsmsConnectionState::Selected);
//                         } else {
//                             // select.req 이외 수신 시 연결 종료
//                             log::error!(
//                                 "control not allowed, disconnect connection. state = {:?}, cont = {:?}",
//                                 self.state,
//                                 header
//                             );
//                             self.emit(HsmsConnEffect::Disconnect);
//                         }
//                     }
//                 }
//             }
//             HsmsConnSignal::TcpConnected => {
//                 log::warn!("tcp already connected");
//             }
//             HsmsConnSignal::TcpDisconnected => {
//                 self.change_state(HsmsConnectionState::NotConnected);
//             }
//             HsmsConnSignal::Timeout(unit) => {
//                 match self.role {
//                     Active => {
//                         // control 중 T6 발생 or TCP 통신 중 T8 발생 -> TCP 커넥션 종료
//                         if matches!(unit, SecsTimeoutUnit::T6 | SecsTimeoutUnit::T8) {
//                             self.emit(HsmsConnEffect::Disconnect);
//                         }
//                     }
//                     Passive => {
//                         // select.req 대기 중 T7 발생 or TCP 통신 중 T8 발생 -> TCP 커넥션 종료
//                         if matches!(unit, SecsTimeoutUnit::T7 | SecsTimeoutUnit::T8) {
//                             self.emit(HsmsConnEffect::Disconnect);
//                         }
//                     }
//                 }
//             }
//         }

//         Ok(())
//     }

//     fn handle_selected(&mut self, signal: HsmsConnSignal) -> Result<(), SecsTransportError> {
//         match signal {
//             HsmsConnSignal::RecvControl(header) => {
//                 match self.role {
//                     Active => {}
//                     Passive => {
//                         match header.stype {
//                             HsmsSType::SelectReq => {
//                                 log::warn!("already selected but recv select.req");
//                                 let select_rsp = HsmsHeader::control(
//                                     0,
//                                     HsmsSelectStatus::AlreadyActive as u8,
//                                     HsmsSType::SelectRsp,
//                                     header.system_byte,
//                                 );
//                                 // 이미 select 상태임을 알림
//                                 self.emit(HsmsConnEffect::SendControl(select_rsp));
//                             }
//                             HsmsSType::DeselectReq | HsmsSType::DeselectRsp => {
//                                 log::warn!("control {:?} not used in E37.1", header.stype);
//                             }
//                             // 상대방이 linktest 요청
//                             HsmsSType::LinktestReq => {
//                                 let linktest_rsp = HsmsHeader::control(
//                                     0,
//                                     HsmsSelectStatus::AlreadyActive as u8,
//                                     HsmsSType::LinktestRsp,
//                                     header.system_byte,
//                                 );
//                                 // linktest 전달
//                                 self.emit(HsmsConnEffect::SendControl(linktest_rsp));
//                             }
//                             HsmsSType::LinktestRsp => {
//                                 // linktest에 대한 T6 timeout 초기화
//                                 self.emit(HsmsConnEffect::ClearTimeout(SecsTimeoutUnit::T6));
//                                 log::info!("linktest success");
//                             }
//                             HsmsSType::SeparateReq => {
//                                 // seperate 요청 받음 -> disconnect 요청
//                                 self.emit(HsmsConnEffect::Disconnect);
//                             }
//                             HsmsSType::RejectReq => {}
//                             _ => {
//                                 // 이상한 데이터를 수신한 경우
//                                 // ex passive인데 select response 수신
//                                 log::error!(
//                                     "invalid control detected, disconnect. {:?}",
//                                     header.stype
//                                 );
//                             }
//                         }
//                     }
//                 }
//             }
//             HsmsConnSignal::TcpConnected => {
//                 log::warn!("tcp already connected");
//             }
//             HsmsConnSignal::TcpDisconnected => {
//                 self.change_state(HsmsConnectionState::NotConnected);
//             }
//             HsmsConnSignal::Timeout(unit) => {
//                 // control 중 T6 발생 or TCP 통신 중 T8 발생 -> TCP 커넥션 종료
//                 if matches!(unit, SecsTimeoutUnit::T6 | SecsTimeoutUnit::T8) {
//                     self.emit(HsmsConnEffect::Disconnect);
//                 }
//             }
//         }

//         Ok(())
//     }

//     fn change_state(&mut self, state: HsmsConnectionState) {
//         log::debug!("state changed from {:?} to {:?}", self.state, state);
//         self.state = state;
//     }

//     /// effect를 호출
//     fn emit(&mut self, effect: HsmsConnEffect) {
//         self.effects.push_back(effect);
//     }

//     pub fn poll_effects(&mut self) -> Vec<HsmsConnEffect> {
//         self.effects.drain(..).collect()
//     }
// }

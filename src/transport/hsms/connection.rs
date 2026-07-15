/// HSMS connection state을 설명
pub enum HsmsConnectionState {
    /// TCP/IP 미연결 상태
    NotConnected,
    // Connected, -> NotSelected / Selected가 이미 의미 내포
    /// TCP/IP 연결 but hsms session 수립 X
    NotSelected,
    /// hsms session 수립된 상태
    Selected,
}

impl HsmsConnectionState {
    pub fn is_connected(&self) -> bool {
        !matches!(self, Self::NotConnected)
    }

    pub fn is_selected(&self) -> bool {
        matches!(self, Self::Selected)
    }
}

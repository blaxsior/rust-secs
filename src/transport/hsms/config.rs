use core::time::Duration;
use core::net::{SocketAddr};

use crate::transport::{ConnectionRole, DeviceId};



///
/// HSMS 통신 구성 시 사용하는 설정
///
pub struct HsmsTransportConfig {
    /// 장치 식별자. 통신 장치의 식별 번호
    pub device_id: DeviceId,
    /// 통신 모드(ACTIVE / PASSIVE)
    pub connection_mode: ConnectionRole,

    /// reply timeout: Send 후 Recv까지 대기 시간
    pub t3_timeout: Duration,
    /// connection seperation timeout: 연속 커넥션 시도 간 대기 시간(간격)
    pub t5_timeout: Duration,
    /// control transaction timeout: control transaction 실패 감지까지 시간
    pub t6_timeout: Duration,
    /// TCP/IP 커넥션이 not selected 상태로 유지 가능한 시간
    pub t7_timeout: Duration,
    /// 단일 HSMS 메시지 내 연속 byte 사이 최대 시간 간격
    pub t8_timeout: Duration,
    
    /// passive mode -> server  주소
    pub local_addr: SocketAddr,
    /// active mode -> 상대 엔티티 주소
    pub remote_addr: SocketAddr,
    
}
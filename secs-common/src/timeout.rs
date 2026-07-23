use crate::TransactionKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimeoutId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecsTimeoutUnit {
    /// Inter Character timeout
    T1,
    /// protocol timeout ENQ-EOT / EOT-length / 2nd checksum-any char
    T2,
    /// send - recv timeout
    T3(TransactionKey),
    /// inter block timeout
    T4(TransactionKey),
    /// connection seperation timeout: 연속 커넥션 시도 간 대기 시간
    T5,
    /// control transaction timeout: control transaction msg 전송 후 응답 대기하는 시간
    T6,
    /// not selected timeout: TCP/IP 커넥션 연결 후 select.req 받을 때까지 시간
    T7,
    /// 단일 HSMS 메시지 내 연속 byte 사이 최대 간격
    T8,
}

impl SecsTimeoutUnit {
    pub fn to_transaction_key(&self) -> TransactionKey {
        match *self {
            SecsTimeoutUnit::T3(key) => key,
            SecsTimeoutUnit::T4(key) => key,
            _ => panic!("unexpected condition {:?}", self),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeoutTicket {
    pub id: TimeoutId,
    pub timeout: SecsTimeoutUnit,
}

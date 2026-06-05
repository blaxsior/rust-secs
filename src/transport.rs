pub mod error;
pub mod secs1;

// 현재 transaction ID 값
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TransactionId(pub u32);

impl TransactionId {
    /// 다음 트랜잭션 id를 얻는다
    pub fn next(&self) -> Self {
        // id = 0인 케이스는 제외
        TransactionId(self.0.wrapping_add(1).max(1))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecsTimeoutUnit {
    /// inter character timeout: 메시지 내 문자 간 timeout
    T1,
    /// protocol timeout: 제어문자 보낸 후 응답 제어문자 도착
    T2,
    /// reply timeout: Primary Message 송신 ~ Secondary Message 수신
    T3(TransactionId),
    /// inter block timeout: 멀티 블록 전송 시 block 수신 간 간격
    T4(TransactionId),
}

///
/// SECS 통신 시 역할
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionMode {
    /// 요청을 시도하는 측 (= master / host)
    Active, // = Master
    /// 요청에 응답하는 측 (= slave / eqp)
    Passive,
}
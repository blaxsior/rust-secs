pub mod error;
pub mod secs1;

/// 현재 transaction ID 값
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SystemByte(pub u32);

impl SystemByte {
    /// 다음 트랜잭션 id를 얻는다
    pub fn next(&self) -> Self {
        // id = 0인 케이스는 제외
        SystemByte(self.0.wrapping_add(1).max(1))
    }
}

/// 트랜잭션을 시작한 쪽(Primary Message Sender 측)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TransactionOwner {
    /// 현재 장치
    Local,
    /// 상대측
    Remote,
}

/// 각 트랜잭션을 구분하는 식별 정보. 트랜잭션 주도측 + system byte(source_id[16] / transaction_id[16])
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TransactionKey {
    pub owner: TransactionOwner,
    pub system_byte: SystemByte,
}

impl TransactionKey {
    pub fn next(&self) -> Self {
        Self {
            owner: self.owner,
            system_byte: self.system_byte.next(),
        }
    }

    pub fn new(owner: TransactionOwner, system_byte: SystemByte) -> Self {
        Self { owner, system_byte }
    }

    pub fn from(role: ConnectionRole, rbit: Rbit, system_byte: SystemByte) -> Self {
        let owner = match (role, rbit) {
            (ConnectionRole::Active, Rbit(false)) => TransactionOwner::Local,
            (ConnectionRole::Active, Rbit(true)) => TransactionOwner::Remote,
            (ConnectionRole::Passive, Rbit(false)) => TransactionOwner::Remote,
            (ConnectionRole::Passive, Rbit(true)) => TransactionOwner::Local,
        };
        Self::new(owner, system_byte)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecsTimeoutUnit {
    /// inter character timeout: 메시지 내 문자 간 timeout
    T1,
    /// protocol timeout: 제어문자 보낸 후 응답 제어문자 도착
    T2,
    /// reply timeout: Primary Message 송신 ~ Secondary Message 수신
    T3(TransactionKey),
    /// inter block timeout: 멀티 블록 전송 시 block 수신 간 간격
    T4(TransactionKey),
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

///
/// SECS 통신 시 역할
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConnectionRole {
    /// 요청을 시도하는 측 (= master / host)
    Active, // = Master
    /// 요청에 응답하는 측 (= slave / eqp)
    Passive,
}

/// SECS 통신 시 사용되는 Device Id
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceId(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rbit(bool);
impl Rbit {
    pub const FORWARD: Self = Self(false);
    pub const REVERSE: Self = Self(true);

    /// 보수 값을 반환
    pub const fn complement(self) -> Self {
        Self(!self.0)
    }
}
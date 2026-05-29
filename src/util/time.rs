use crate::transport::error::SecsTimeoutUnit;
use alloc::collections::BTreeMap;

/// 각 타임아웃을 식별하기 위한 키
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimeoutId(u64);

/// 타임아웃 발생을 표현하는 티켓
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeoutTicket {
    pub id: TimeoutId,
    pub timeout: SecsTimeoutUnit,
}

pub struct TimeoutIdGenerator {
    next: u64,
}

impl TimeoutIdGenerator {
    pub fn generate(&mut self) -> TimeoutId {
        let id = TimeoutId(self.next);
        self.next = self.next.wrapping_add(1);
        id
    }
}

/// 타임아웃을 다루는 객체
pub struct TimeoutManager {
    id_generator: TimeoutIdGenerator,
    active: BTreeMap<SecsTimeoutUnit, TimeoutId>,
}

impl TimeoutManager {
    pub fn new() -> Self {
        Self {
            id_generator: TimeoutIdGenerator { next: 0 },
            active: BTreeMap::new(),
        }
    }

    /// timeout을 발행한다.
    pub fn issue(&mut self, timeout: SecsTimeoutUnit) -> TimeoutTicket {
        let id: TimeoutId = self.id_generator.generate();
        self.active.insert(timeout, id);
        TimeoutTicket { id, timeout }
    }

    /// timeout을 취소한다.
    pub fn cancel(&mut self, target: SecsTimeoutUnit) -> bool {
        self.active.remove(&target).is_some()
    }

    /// timeout이 아직 활성 상태인지 확인한다.
    fn is_active(&self, ticket: &TimeoutTicket) -> bool {
        self.active
            .get(&ticket.timeout)
            .is_some_and(|timeout_id| *timeout_id == ticket.id)
    }

    /// timeout 발생 처리.
    ///
    /// timeout이 유효하면 true를 반환하고 제거한다.
    /// 이미 취소되었거나 다른 timeout으로 재사용된 경우 false를 반환한다.
    pub fn fire(&mut self, ticket: &TimeoutTicket) -> bool {
        if self.is_active(ticket) {
            self.active.remove(&ticket.timeout);
            true
        } else {
            false
        }
    }
}

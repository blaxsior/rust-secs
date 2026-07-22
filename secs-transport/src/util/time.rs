use alloc::collections::BTreeMap;

use crate::transport::{SecsTimeoutUnit, TimeoutId, TimeoutTicket};

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

pub struct TimeoutManager {
    id_generator: TimeoutIdGenerator,
    active: BTreeMap<SecsTimeoutUnit, TimeoutTicket>,
}

impl TimeoutManager {
    pub fn new() -> Self {
        Self {
            id_generator: TimeoutIdGenerator { next: 0 },
            active: BTreeMap::new(),
        }
    }

    pub fn issue(&mut self, timeout: SecsTimeoutUnit) -> TimeoutTicket {
        let id = self.id_generator.generate();
        let ticket = TimeoutTicket { id, timeout };
        self.active.insert(timeout, ticket);
        ticket
    }

    pub fn cancel(&mut self, target: SecsTimeoutUnit) -> bool {
        self.active.remove(&target).is_some()
    }

    fn is_active(&self, ticket: &TimeoutTicket) -> bool {
        self.active
            .get(&ticket.timeout)
            .is_some_and(|t| *t == *ticket)
    }

    pub fn fire(&mut self, ticket: &TimeoutTicket) -> bool {
        if self.is_active(ticket) {
            self.active.remove(&ticket.timeout);
            true
        } else {
            false
        }
    }
}

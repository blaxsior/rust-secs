use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashMap};
use std::convert::Infallible;
use std::time::{Duration, Instant};

use secs_runtime_core::{SecsTimer, TimeoutId, TimeoutTicket};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TimerEntry {
    deadline: Instant,
    id: TimeoutId,
}

impl Ord for TimerEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.deadline
            .cmp(&other.deadline)
            .then_with(|| self.id.cmp(&other.id))
    }
}

impl PartialOrd for TimerEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct StdSecsTimer {
    queue: BinaryHeap<Reverse<TimerEntry>>,
    tickets: HashMap<TimeoutId, (Instant, TimeoutTicket)>,
}

impl StdSecsTimer {
    pub fn new() -> Self {
        Self {
            queue: BinaryHeap::new(),
            tickets: HashMap::new(),
        }
    }
}

impl Default for StdSecsTimer {
    fn default() -> Self {
        Self::new()
    }
}

impl SecsTimer for StdSecsTimer {
    type Error = Infallible;
    type Duration = Duration;
    type Handle = TimeoutId;

    fn start_timeout(
        &mut self,
        ticket: TimeoutTicket,
        duration: Self::Duration,
    ) -> Result<Self::Handle, Self::Error> {
        let deadline = Instant::now() + duration;
        let id = ticket.id;

        self.queue.push(Reverse(TimerEntry { deadline, id }));
        self.tickets.insert(id, (deadline, ticket));

        Ok(id)
    }

    fn poll_timeout(&mut self) -> Result<Option<TimeoutTicket>, Self::Error> {
        let now = Instant::now();

        while let Some(Reverse(entry)) = self.queue.peek().copied() {
            if entry.deadline > now {
                return Ok(None);
            }

            self.queue.pop();
            let Some((deadline, ticket)) = self.tickets.get(&entry.id).copied() else {
                continue;
            };

            if deadline != entry.deadline {
                continue;
            }

            self.tickets.remove(&entry.id);
            return Ok(Some(ticket));
        }

        Ok(None)
    }
}

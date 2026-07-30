use std::convert::Infallible;

use secs_runtime_core::{SecsTimer, TimeoutId, TimeoutTicket};

#[derive(Debug, Clone, Copy)]
struct TimerEntry {
    deadline_ms: f64,
    ticket: TimeoutTicket,
}

pub struct WebSecsTimer {
    entries: Vec<TimerEntry>,
}

impl WebSecsTimer {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    fn now_ms() -> f64 {
        js_sys::Date::now()
    }
}

impl Default for WebSecsTimer {
    fn default() -> Self {
        Self::new()
    }
}

impl SecsTimer for WebSecsTimer {
    type Error = Infallible;
    type Duration = f64;
    type Handle = TimeoutId;

    fn start_timeout(
        &mut self,
        ticket: TimeoutTicket,
        duration_ms: Self::Duration,
    ) -> Result<Self::Handle, Self::Error> {
        let id = ticket.id;
        self.entries.push(TimerEntry {
            deadline_ms: Self::now_ms() + duration_ms,
            ticket,
        });

        Ok(id)
    }

    fn poll_timeout(&mut self) -> Result<Option<TimeoutTicket>, Self::Error> {
        let now_ms = Self::now_ms();
        let Some(index) = self
            .entries
            .iter()
            .position(|entry| entry.deadline_ms <= now_ms)
        else {
            return Ok(None);
        };

        Ok(Some(self.entries.swap_remove(index).ticket))
    }
}

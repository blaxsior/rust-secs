use alloc::vec::Vec;

use crate::SecsKey;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventEntry {
    reports: Vec<SecsKey>,
    enabled: bool,
}

impl EventEntry {
    pub fn new(reports: Vec<SecsKey>) -> Self {
        Self {
            reports,
            enabled: true,
        }
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    pub fn reports(&self) -> &[SecsKey] {
        &self.reports
    }

    pub fn reports_mut(&mut self) -> &mut Vec<SecsKey> {
        &mut self.reports
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

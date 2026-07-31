use alloc::collections::BTreeMap;

use secs_ii::item::Secs2Variant;

use crate::{EventEntry, ReportEntry, SecsKey, SecsModelError, ValueEntry};

#[derive(Debug, Default)]
pub struct SecsDictionary {
    values: BTreeMap<SecsKey, ValueEntry>,
    reports: BTreeMap<SecsKey, ReportEntry>,
    events: BTreeMap<SecsKey, EventEntry>,
}

impl SecsDictionary {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_value(&mut self, key: impl Into<SecsKey>, entry: ValueEntry) {
        self.values.insert(key.into(), entry);
    }

    pub fn insert_report(&mut self, key: impl Into<SecsKey>, entry: ReportEntry) {
        self.reports.insert(key.into(), entry);
    }

    pub fn insert_event(&mut self, key: impl Into<SecsKey>, entry: EventEntry) {
        self.events.insert(key.into(), entry);
    }

    pub fn value(&self, key: &SecsKey) -> Option<&ValueEntry> {
        self.values.get(key)
    }

    pub fn value_mut(&mut self, key: &SecsKey) -> Option<&mut ValueEntry> {
        self.values.get_mut(key)
    }

    pub fn report(&self, key: &SecsKey) -> Option<&ReportEntry> {
        self.reports.get(key)
    }

    pub fn report_mut(&mut self, key: &SecsKey) -> Option<&mut ReportEntry> {
        self.reports.get_mut(key)
    }

    pub fn event(&self, key: &SecsKey) -> Option<&EventEntry> {
        self.events.get(key)
    }

    pub fn event_mut(&mut self, key: &SecsKey) -> Option<&mut EventEntry> {
        self.events.get_mut(key)
    }

    pub fn read_value(&self, key: &SecsKey) -> Result<Option<&Secs2Variant>, SecsModelError> {
        self.values
            .get(key)
            .map(ValueEntry::value)
            .ok_or_else(|| SecsModelError::UnknownValue(key.clone()))
    }

    pub fn write_value(
        &mut self,
        key: &SecsKey,
        value: Secs2Variant,
    ) -> Result<(), SecsModelError> {
        let entry = self
            .values
            .get_mut(key)
            .ok_or_else(|| SecsModelError::UnknownValue(key.clone()))?;

        if !entry.is_writable() {
            return Err(SecsModelError::ReadOnlyValue(key.clone()));
        }

        entry.set_value(value);
        Ok(())
    }
}

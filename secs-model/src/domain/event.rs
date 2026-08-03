use alloc::{borrow::ToOwned, collections::BTreeMap, string::String, vec::Vec};
use core::fmt;

use serde::{Deserialize, Serialize};

use crate::{EventRepository, NoopEventRepository, ReportId, SecsModelError, StoreError};

#[repr(transparent)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EventId(String);

impl EventId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Debug for EventId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ECID: ").field(&self.0).finish()
    }
}

impl From<String> for EventId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for EventId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct EventSpec {
    pub id: EventId,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    pub reports: Vec<ReportId>,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "crate::domain::default_policy::persistent")]
    pub persistent: bool,
    #[serde(default = "crate::domain::default_policy::readonly")]
    pub readonly: bool,
}

impl EventSpec {
    pub fn new(id: EventId, reports: Vec<ReportId>) -> Self {
        Self {
            id,
            name: None,
            description: None,
            reports,
            enabled: false,
            persistent: false,
            readonly: false,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn persistent(mut self) -> Self {
        self.persistent = true;
        self
    }

    pub fn readonly(mut self) -> Self {
        self.readonly = true;
        self
    }

    pub fn reports(&self) -> &[ReportId] {
        &self.reports
    }

    pub fn reports_mut(&mut self) -> &mut Vec<ReportId> {
        &mut self.reports
    }

    pub fn has_report(&self, report: &ReportId) -> bool {
        self.reports.contains(report)
    }

    pub fn link_report(&mut self, report: ReportId) {
        if !self.reports.contains(&report) {
            self.reports.push(report);
        }
    }

    pub fn unlink_report(&mut self, report: &ReportId) {
        self.reports.retain(|it| it != report);
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

#[derive(Debug)]
pub struct EventDictionary<R> {
    events: BTreeMap<EventId, EventSpec>,
    repository: R,
}

impl<R> EventDictionary<R>
where
    R: EventRepository,
{
    pub fn with_store(repository: R) -> Result<Self, StoreError> {
        let mut dictionary = Self {
            events: BTreeMap::new(),
            repository,
        };
        for spec in dictionary.repository.find_all()? {
            dictionary.insert(spec);
        }

        Ok(dictionary)
    }

    fn insert(&mut self, spec: EventSpec) {
        self.events.insert(spec.id.clone(), spec);
    }

    pub fn save(&mut self, spec: EventSpec) -> Result<(), SecsModelError> {
        if self
            .events
            .get(&spec.id)
            .is_some_and(|current| current.readonly)
        {
            log::warn!("skip event save because event is readonly: {:?}", spec.id);
            return Err(SecsModelError::ReadOnlyEvent(spec.id));
        }

        if spec.persistent {
            if let Err(err) = self.repository.save(&spec) {
                log::error!("failed to save event on repository {:?}", err);
            }
        }
        self.insert(spec);
        Ok(())
    }

    pub fn delete(&mut self, id: &EventId) -> Result<(), SecsModelError> {
        let spec = self.get(id)?;

        if spec.readonly {
            log::warn!("skip event delete because event is readonly: {:?}", id);
            return Err(SecsModelError::ReadOnlyEvent(id.clone()));
        }

        if spec.persistent {
            if let Err(err) = self.repository.delete(id) {
                log::error!("failed to remove event from repository {:?}", err);
            }
        }
        self.events.remove(id);
        Ok(())
    }

    pub fn get(&self, id: &EventId) -> Result<&EventSpec, SecsModelError> {
        self.events
            .get(id)
            .ok_or_else(|| SecsModelError::UnknownEvent(id.clone()))
    }

    pub fn get_mut(&mut self, id: &EventId) -> Result<&mut EventSpec, SecsModelError> {
        self.events
            .get_mut(id)
            .ok_or_else(|| SecsModelError::UnknownEvent(id.clone()))
    }

    pub fn link_report(&mut self, event: &EventId, report: ReportId) -> Result<(), SecsModelError> {
        let spec = self
            .events
            .get_mut(event)
            .ok_or_else(|| SecsModelError::UnknownEvent(event.clone()))?;

        if spec.readonly {
            log::warn!("skip report link because event is readonly: {:?}", event);
            return Err(SecsModelError::ReadOnlyEvent(event.clone()));
        }

        spec.link_report(report);
        if spec.persistent {
            if let Err(err) = self.repository.save(spec) {
                log::error!("failed to save event on repository {:?}", err);
            }
        }
        Ok(())
    }

    pub fn unlink_report(&mut self, report: &ReportId) -> Result<(), SecsModelError> {
        for event in self.events.values_mut() {
            if !event.has_report(report) {
                continue;
            }

            if event.readonly {
                log::warn!("skip report unlink because event is readonly: {:?}", event.id);
                return Err(SecsModelError::ReadOnlyEvent(event.id.clone()));
            }

            event.unlink_report(report);
            if event.persistent {
                if let Err(err) = self.repository.save(event) {
                    log::error!("failed to save event on repository {:?}", err);
                }
            }
        }

        Ok(())
    }

    pub fn set_enabled(&mut self, event: &EventId, enabled: bool) -> Result<(), SecsModelError> {
        let spec = self
            .events
            .get_mut(event)
            .ok_or_else(|| SecsModelError::UnknownEvent(event.clone()))?;

        if spec.readonly {
            log::warn!("skip event enabled change because event is readonly: {:?}", event);
            return Err(SecsModelError::ReadOnlyEvent(event.clone()));
        }

        spec.set_enabled(enabled);
        if spec.persistent {
            if let Err(err) = self.repository.save(spec) {
                log::error!("failed to save event on repository {:?}", err);
            }
        }
        Ok(())
    }
}

impl EventDictionary<NoopEventRepository> {
    pub fn new() -> Self {
        Self {
            events: BTreeMap::new(),
            repository: NoopEventRepository,
        }
    }
}

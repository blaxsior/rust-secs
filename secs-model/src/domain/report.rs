use alloc::{borrow::ToOwned, collections::BTreeMap, string::String, vec::Vec};
use core::fmt;

use serde::{Deserialize, Serialize};

use crate::{NoopReportRepository, ReportRepository, SecsModelError, StoreError, ValueId};

#[repr(transparent)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ReportId(String);

impl ReportId {
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

impl fmt::Debug for ReportId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("RPTID: ").field(&self.0).finish()
    }
}

impl From<String> for ReportId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for ReportId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ReportSpec {
    pub id: ReportId,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    pub values: Vec<ValueId>,
    #[serde(default = "crate::domain::default_policy::persistent")]
    pub persistent: bool,
    #[serde(default = "crate::domain::default_policy::readonly")]
    pub readonly: bool,
}

impl ReportSpec {
    pub fn new(id: ReportId, values: Vec<ValueId>) -> Self {
        Self {
            id,
            name: None,
            description: None,
            values,
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

    pub fn values(&self) -> &[ValueId] {
        &self.values
    }

    pub fn values_mut(&mut self) -> &mut Vec<ValueId> {
        &mut self.values
    }
}

#[derive(Debug)]
pub struct ReportDictionary<R> {
    reports: BTreeMap<ReportId, ReportSpec>,
    repository: R,
}

impl<R> ReportDictionary<R>
where
    R: ReportRepository,
{
    pub fn with_store(repository: R) -> Result<Self, StoreError> {
        let mut dictionary = Self {
            reports: BTreeMap::new(),
            repository,
        };
        for spec in dictionary.repository.load_all()? {
            dictionary.insert(spec);
        }

        Ok(dictionary)
    }

    fn insert(&mut self, spec: ReportSpec) {
        self.reports.insert(spec.id.clone(), spec);
    }

    pub fn save(&mut self, spec: ReportSpec) -> Result<(), SecsModelError> {
        if self
            .reports
            .get(&spec.id)
            .is_some_and(|current| current.readonly)
        {
            log::warn!("skip report save because report is readonly: {:?}", spec.id);
            return Err(SecsModelError::ReadOnlyReport(spec.id));
        }

        if spec.persistent {
            if let Err(err) = self.repository.save(&spec) {
                log::error!("failed to save report on repository {:?}", err);
            }
        }
        self.insert(spec);
        Ok(())
    }

    pub fn remove(&mut self, id: &ReportId) -> Result<(), SecsModelError> {
        let spec = self.get(id)?;

        if spec.readonly {
            log::warn!("skip report delete because report is readonly: {:?}", id);
            return Err(SecsModelError::ReadOnlyReport(id.clone()));
        }

        if spec.persistent {
            if let Err(err) = self.repository.remove(id) {
                log::error!("failed to remove report from repository {:?}", err);
            }
        }
        self.reports.remove(id);
        Ok(())
    }

    pub fn get(&self, id: &ReportId) -> Result<&ReportSpec, SecsModelError> {
        self.reports
            .get(id)
            .ok_or_else(|| SecsModelError::UnknownReport(id.clone()))
    }

    pub fn get_mut(&mut self, id: &ReportId) -> Result<&mut ReportSpec, SecsModelError> {
        self.reports
            .get_mut(id)
            .ok_or_else(|| SecsModelError::UnknownReport(id.clone()))
    }
}

impl ReportDictionary<NoopReportRepository> {
    pub fn new() -> Self {
        Self {
            reports: BTreeMap::new(),
            repository: NoopReportRepository,
        }
    }
}

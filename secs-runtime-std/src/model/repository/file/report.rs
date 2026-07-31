use secs_model::{ReportId, ReportRepository, ReportSpec, StoreError};
use secs_runtime_core::DataStore;

use crate::model::{
    file::codec::ModelCodec,
    repository::file::{load_all, remove, upsert},
};

#[derive(Debug)]
pub struct ReportFileRepository<S, C> {
    store: S,
    codec: C,
    key: String,
}

impl<S, C> ReportFileRepository<S, C> {
    pub fn new(store: S, codec: C, key: impl Into<String>) -> Self {
        Self {
            store,
            codec,
            key: key.into(),
        }
    }
}

impl<S, C> ReportRepository for ReportFileRepository<S, C>
where
    S: DataStore,
    C: ModelCodec<ReportSpec>,
{
    fn load_all(&mut self) -> Result<Vec<ReportSpec>, StoreError> {
        load_all(&mut self.store, &self.codec, &self.key)
    }

    fn save(&mut self, spec: &ReportSpec) -> Result<(), StoreError> {
        upsert(
            &mut self.store,
            &self.codec,
            &self.key,
            spec.clone(),
            |left, right| left.id == right.id,
        )
    }

    fn remove(&mut self, id: &ReportId) -> Result<(), StoreError> {
        remove(&mut self.store, &self.codec, &self.key, |spec: &ReportSpec| {
            &spec.id == id
        })
    }
}

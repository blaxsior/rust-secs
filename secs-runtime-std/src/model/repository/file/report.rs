use secs_model::{ReportId, ReportRepository, ReportSpec, StoreError};
use secs_runtime_core::DataStore;

#[derive(Debug)]
pub struct ReportFileRepository<S> {
    store: S,
}

impl<S> ReportFileRepository<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S> ReportRepository for ReportFileRepository<S>
where
    S: DataStore<ReportSpec>,
{
    fn find_all(&mut self) -> Result<Vec<ReportSpec>, StoreError> {
        self.store
            .find_all()
            .map_err(|_| StoreError::LoadFailed)
    }

    fn save(&mut self, spec: &ReportSpec) -> Result<(), StoreError> {
        self.store
            .save(spec.id.as_str(), spec)
            .map_err(|_| StoreError::SaveFailed)
    }

    fn delete(&mut self, id: &ReportId) -> Result<(), StoreError> {
        self.store
            .delete(id.as_str())
            .map_err(|_| StoreError::DeleteFailed)
    }
}

use secs_model::{
    StoreError, ValueData, ValueDataRepository, ValueId, ValueSpec, ValueSpecRepository,
};
use secs_runtime_core::DataStore;

#[derive(Debug)]
pub struct ValueSpecFileRepository<S> {
    store: S,
}

impl<S> ValueSpecFileRepository<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S> ValueSpecRepository for ValueSpecFileRepository<S>
where
    S: DataStore<ValueSpec>,
{
    fn find_all(&mut self) -> Result<Vec<ValueSpec>, StoreError> {
        self.store
            .find_all()
            .map_err(|_| StoreError::LoadFailed)
    }
}

#[derive(Debug)]
pub struct ValueDataFileRepository<S> {
    store: S,
}

impl<S> ValueDataFileRepository<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S> ValueDataRepository for ValueDataFileRepository<S>
where
    S: DataStore<ValueData>,
{
    fn find_all(&mut self) -> Result<Vec<ValueData>, StoreError> {
        self.store
            .find_all()
            .map_err(|_| StoreError::LoadFailed)
    }

    fn save(&mut self, data: &ValueData) -> Result<(), StoreError> {
        self.store
            .save(data.id.as_str(), data)
            .map_err(|_| StoreError::SaveFailed)
    }

    fn delete(&mut self, id: &ValueId) -> Result<(), StoreError> {
        self.store
            .delete(id.as_str())
            .map_err(|_| StoreError::DeleteFailed)
    }
}

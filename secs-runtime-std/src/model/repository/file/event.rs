use secs_model::{EventId, EventRepository, EventSpec, StoreError};
use secs_runtime_core::DataStore;

#[derive(Debug)]
pub struct EventFileRepository<S> {
    store: S,
}

impl<S> EventFileRepository<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S> EventRepository for EventFileRepository<S>
where
    S: DataStore<EventSpec>,
{
    fn find_all(&mut self) -> Result<Vec<EventSpec>, StoreError> {
        self.store
            .find_all()
            .map_err(|_| StoreError::LoadFailed)
    }

    fn save(&mut self, spec: &EventSpec) -> Result<(), StoreError> {
        self.store
            .save(spec.id.as_str(), spec)
            .map_err(|_| StoreError::SaveFailed)
    }

    fn delete(&mut self, id: &EventId) -> Result<(), StoreError> {
        self.store
            .delete(id.as_str())
            .map_err(|_| StoreError::DeleteFailed)
    }
}

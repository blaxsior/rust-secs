use secs_model::{EventId, EventRepository, EventSpec, StoreError};
use secs_runtime_core::DataStore;

use crate::model::{
    file::codec::ModelCodec,
    repository::file::{load_all, remove, upsert},
};

#[derive(Debug)]
pub struct EventFileRepository<S, C> {
    store: S,
    codec: C,
    key: String,
}

impl<S, C> EventFileRepository<S, C> {
    pub fn new(store: S, codec: C, key: impl Into<String>) -> Self {
        Self {
            store,
            codec,
            key: key.into(),
        }
    }
}

impl<S, C> EventRepository for EventFileRepository<S, C>
where
    S: DataStore,
    C: ModelCodec<EventSpec>,
{
    fn load_all(&mut self) -> Result<Vec<EventSpec>, StoreError> {
        load_all(&mut self.store, &self.codec, &self.key)
    }

    fn save(&mut self, spec: &EventSpec) -> Result<(), StoreError> {
        upsert(
            &mut self.store,
            &self.codec,
            &self.key,
            spec.clone(),
            |left, right| left.id == right.id,
        )
    }

    fn remove(&mut self, id: &EventId) -> Result<(), StoreError> {
        remove(&mut self.store, &self.codec, &self.key, |spec: &EventSpec| {
            &spec.id == id
        })
    }
}

use secs_model::{
    StoreError, ValueData, ValueDataRepository, ValueId, ValueSpec, ValueSpecRepository,
};
use secs_runtime_core::DataStore;

use crate::model::{
    file::codec::ModelCodec,
    repository::file::{load_all, remove, upsert},
};

#[derive(Debug)]
pub struct ValueSpecFileRepository<S, C> {
    store: S,
    codec: C,
    key: String,
}

impl<S, C> ValueSpecFileRepository<S, C> {
    pub fn new(store: S, codec: C, key: impl Into<String>) -> Self {
        Self {
            store,
            codec,
            key: key.into(),
        }
    }
}

impl<S, C> ValueSpecRepository for ValueSpecFileRepository<S, C>
where
    S: DataStore,
    C: ModelCodec<ValueSpec>,
{
    fn load_all(&mut self) -> Result<Vec<ValueSpec>, StoreError> {
        load_all(&mut self.store, &self.codec, &self.key)
    }
}

#[derive(Debug)]
pub struct ValueDataFileRepository<S, C> {
    store: S,
    codec: C,
    key: String,
}

impl<S, C> ValueDataFileRepository<S, C> {
    pub fn new(store: S, codec: C, key: impl Into<String>) -> Self {
        Self {
            store,
            codec,
            key: key.into(),
        }
    }
}

impl<S, C> ValueDataRepository for ValueDataFileRepository<S, C>
where
    S: DataStore,
    C: ModelCodec<ValueData>,
{
    fn load_all(&mut self) -> Result<Vec<ValueData>, StoreError> {
        load_all(&mut self.store, &self.codec, &self.key)
    }

    fn save(&mut self, data: &ValueData) -> Result<(), StoreError> {
        upsert(
            &mut self.store,
            &self.codec,
            &self.key,
            data.clone(),
            |left, right| left.id == right.id,
        )
    }

    fn remove(&mut self, id: &ValueId) -> Result<(), StoreError> {
        remove(&mut self.store, &self.codec, &self.key, |data: &ValueData| {
            &data.id == id
        })
    }
}

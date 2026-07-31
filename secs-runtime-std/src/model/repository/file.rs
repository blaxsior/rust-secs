pub mod event;
pub mod report;
pub mod value;

use secs_model::StoreError;
use secs_runtime_core::DataStore;

use crate::model::file::codec::ModelCodec;

pub use event::EventFileRepository;
pub use report::ReportFileRepository;
pub use value::{ValueDataFileRepository, ValueSpecFileRepository};

pub(crate) fn load_all<S, C, T>(
    store: &mut S,
    codec: &C,
    key: &str,
) -> Result<Vec<T>, StoreError>
where
    S: DataStore,
    C: ModelCodec<T>,
{
    let Some(bytes) = store.load(key).map_err(|_| StoreError::LoadFailed)? else {
        return Ok(Vec::new());
    };

    codec.decode(&bytes).map_err(|_| StoreError::LoadFailed)
}

pub(crate) fn save_all<S, C, T>(
    store: &mut S,
    codec: &C,
    key: &str,
    items: &[T],
) -> Result<(), StoreError>
where
    S: DataStore,
    C: ModelCodec<T>,
{
    let bytes = codec.encode(items).map_err(|_| StoreError::SaveFailed)?;
    store
        .save(key, &bytes)
        .map_err(|_| StoreError::SaveFailed)
}

pub(crate) fn upsert<S, C, T, F>(
    store: &mut S,
    codec: &C,
    key: &str,
    item: T,
    mut is_same: F,
) -> Result<(), StoreError>
where
    S: DataStore,
    C: ModelCodec<T>,
    F: FnMut(&T, &T) -> bool,
{
    let mut items = load_all(store, codec, key)?;

    match items.iter_mut().find(|current| is_same(current, &item)) {
        Some(current) => *current = item,
        None => items.push(item),
    }

    save_all(store, codec, key, &items)
}

pub(crate) fn remove<S, C, T, F>(
    store: &mut S,
    codec: &C,
    key: &str,
    mut should_remove: F,
) -> Result<(), StoreError>
where
    S: DataStore,
    C: ModelCodec<T>,
    F: FnMut(&T) -> bool,
{
    let mut items = load_all(store, codec, key)?;
    items.retain(|item| !should_remove(item));
    save_all(store, codec, key, &items)
}

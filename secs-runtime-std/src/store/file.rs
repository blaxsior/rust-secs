use std::{collections::BTreeMap, fs, io, path::PathBuf};

use secs_runtime_core::DataStore;

use crate::model::file::codec::ModelCodec;

#[derive(Debug, thiserror::Error)]
pub enum FileStoreError {
    #[error("failed to read file data")]
    LoadFailed(#[source] io::Error),
    #[error("failed to decode file data: {0}")]
    DecodeFailed(String),
    #[error("failed to encode file data: {0}")]
    EncodeFailed(String),
    #[error("failed to write file data")]
    SaveFailed(#[source] io::Error),
    #[error("failed to remove file data")]
    RemoveFailed(#[source] io::Error),
}

#[derive(Debug, Clone)]
pub struct FileStore<C, T> {
    path: PathBuf,
    codec: C,
    items: BTreeMap<String, T>,
}

pub type FileDataStore<C, T> = FileStore<C, T>;
pub type FileDataStoreError = FileStoreError;

impl<C, T> FileStore<C, T>
where
    C: ModelCodec<T>,
    C::Error: core::fmt::Display,
{
    pub fn new(path: impl Into<PathBuf>, codec: C) -> Result<Self, FileStoreError> {
        let mut store = Self {
            path: path.into(),
            codec,
            items: BTreeMap::new(),
        };
        store.load()?;
        Ok(store)
    }

    pub fn load(&mut self) -> Result<(), FileStoreError> {
        match fs::read(&self.path) {
            Ok(bytes) if bytes.is_empty() => {
                self.items.clear();
                Ok(())
            }
            Ok(bytes) => {
                self.items = self
                    .codec
                    .decode(&bytes)
                    .map_err(|error| FileStoreError::DecodeFailed(error.to_string()))?;
                Ok(())
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                self.items.clear();
                Ok(())
            }
            Err(error) => Err(FileStoreError::LoadFailed(error)),
        }
    }
}

impl<C, T> DataStore<T> for FileStore<C, T>
where
    C: ModelCodec<T>,
    C::Error: core::fmt::Display,
    T: Clone,
{
    type Error = FileStoreError;

    fn load(&mut self) -> Result<(), Self::Error> {
        Self::load(self)
    }

    fn find(&mut self, key: &str) -> Result<Option<T>, Self::Error> {
        Ok(self.items.get(key).cloned())
    }

    fn find_all(&mut self) -> Result<Vec<T>, Self::Error> {
        Ok(self.items.values().cloned().collect())
    }

    fn save(&mut self, key: &str, item: &T) -> Result<(), Self::Error> {
        self.items.insert(key.to_owned(), item.clone());
        self.save_all()
    }

    fn save_all(&self) -> Result<(), FileStoreError> {
        let bytes = self
            .codec
            .encode(&self.items)
            .map_err(|error| FileStoreError::EncodeFailed(error.to_string()))?;
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(FileStoreError::SaveFailed)?;
        }

        fs::write(&self.path, bytes).map_err(FileStoreError::SaveFailed)
    }

    fn delete(&mut self, key: &str) -> Result<(), Self::Error> {
        self.items.remove(key);
        self.save_all()
    }
}

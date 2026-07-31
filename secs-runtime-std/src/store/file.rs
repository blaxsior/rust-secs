use std::{
    fs,
    io,
    path::{Path, PathBuf},
};

use secs_runtime_core::DataStore;

#[derive(Debug, thiserror::Error)]
pub enum FileDataStoreError {
    #[error("failed to read file data")]
    LoadFailed(#[source] io::Error),
    #[error("failed to write file data")]
    SaveFailed(#[source] io::Error),
    #[error("failed to remove file data")]
    RemoveFailed(#[source] io::Error),
}

#[derive(Debug, Clone)]
pub struct FileDataStore {
    root: PathBuf,
}

impl FileDataStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn resolve(&self, key: &str) -> PathBuf {
        self.root.join(Path::new(key))
    }
}

impl DataStore for FileDataStore {
    type Error = FileDataStoreError;

    fn load(&mut self, key: &str) -> Result<Option<Vec<u8>>, Self::Error> {
        let path = self.resolve(key);
        match fs::read(path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(FileDataStoreError::LoadFailed(error)),
        }
    }

    fn save(&mut self, key: &str, bytes: &[u8]) -> Result<(), Self::Error> {
        let path = self.resolve(key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(FileDataStoreError::SaveFailed)?;
        }

        fs::write(path, bytes).map_err(FileDataStoreError::SaveFailed)
    }

    fn remove(&mut self, key: &str) -> Result<(), Self::Error> {
        let path = self.resolve(key);
        match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(FileDataStoreError::RemoveFailed(error)),
        }
    }
}

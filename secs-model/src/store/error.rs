#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum StoreError {
    #[error("failed to load specs")]
    LoadFailed,

    #[error("failed to save spec")]
    SaveFailed,

    #[error("failed to delete spec")]
    DeleteFailed,

    #[error("store operation is not supported")]
    Unsupported,
}

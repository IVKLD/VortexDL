use thiserror::Error;

#[derive(Debug, Error)]
pub enum AdbError {
    #[error("adb binary not found in PATH")]
    NotAvailable,
    #[error("Sync already in progress")]
    AlreadyInProgress,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

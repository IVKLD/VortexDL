use thiserror::Error;

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Authentication failed")]
    Auth,

    #[error("Remote resource not found")]
    NotFound,

    #[error("HTTP error: status {0}")]
    HttpStatus(reqwest::StatusCode),

    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    Database(#[from] anyhow::Error),

    #[error("Operation not supported: {0}")]
    OperationNotSupported(String),
}

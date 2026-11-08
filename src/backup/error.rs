use std::io;

use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::api::errors::ApiError;

#[derive(Debug, Error)]
pub enum BackupError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Authentication failed: invalid credentials or unauthorized")]
    Auth,

    #[error("Remote resource not found")]
    NotFound,

    #[error("HTTP error: status {0}")]
    HttpStatus(reqwest::StatusCode),

    #[error("File system error: {0}")]
    FileSystem(#[from] io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    Database(#[from] anyhow::Error),

    #[error("Operation not supported: {0}")]
    OperationNotSupported(String),
}

impl IntoResponse for BackupError {
    fn into_response(self) -> Response {
        ApiError::from(self).into_response()
    }
}

use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::api::errors::ApiError;

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    Database(#[from] redb::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for SettingsError {
    fn into_response(self) -> Response {
        ApiError::from(self).into_response()
    }
}

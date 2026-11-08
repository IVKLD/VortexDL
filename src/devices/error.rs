use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::api::errors::ApiError;

#[derive(Debug, Error)]
pub enum DeviceError {
    #[error("adb binary not found in PATH")]
    NotAvailable,
    #[error("Sync already in progress")]
    AlreadyInProgress,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for DeviceError {
    fn into_response(self) -> Response {
        ApiError::from(self).into_response()
    }
}

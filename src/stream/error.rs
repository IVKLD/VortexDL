use std::{convert::Infallible, io};

use axum::{
    http::Error as HttpError,
    response::{IntoResponse, Response},
};
use thiserror::Error;

use crate::{api::errors::ApiError, providers::ProviderError};

#[derive(Debug, Error)]
pub enum StreamError {
    #[error("Track not found: {0}")]
    TrackNotFound(i64),

    #[error("Unsupported or local music platform for track ID: {0}")]
    UnsupportedPlatform(i64),

    #[error(transparent)]
    Provider(#[from] ProviderError),

    #[error("HTTP error: {0}")]
    Http(#[from] HttpError),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
}

impl From<Infallible> for StreamError {
    fn from(err: Infallible) -> Self {
        match err {}
    }
}

impl IntoResponse for StreamError {
    fn into_response(self) -> Response {
        ApiError::from(self).into_response()
    }
}


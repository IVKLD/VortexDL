use std::io;

use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::{api::errors::ApiError, library::metadata::MetadataError};

#[derive(Debug, Error)]
pub enum LibraryError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error(transparent)]
    Metadata(#[from] MetadataError),

    #[error("ID3 metadata error: {0}")]
    Id3(#[from] id3::Error),

    #[error("Database error: {0}")]
    Database(#[from] redb::Error),

    #[error("Background task error: {0}")]
    TaskJoin(#[from] tokio::task::JoinError),

    #[error("Track not found: {0}")]
    TrackNotFound(i64),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for LibraryError {
    fn into_response(self) -> Response {
        ApiError::from(self).into_response()
    }
}

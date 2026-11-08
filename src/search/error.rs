use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::{api::errors::ApiError, providers::ProviderError};

#[derive(Debug, Error)]
pub enum SearchError {
    #[error("Search query cannot be empty")]
    EmptyQuery,

    #[error(transparent)]
    SoundCloud(#[from] soundcloud_rs::Error),

    #[error(transparent)]
    Provider(#[from] ProviderError),
}

impl IntoResponse for SearchError {
    fn into_response(self) -> Response {
        ApiError::from(self).into_response()
    }
}

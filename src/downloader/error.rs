use std::io;

use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::{api::errors::ApiError, library::LibraryError, providers::ProviderError};

#[derive(Debug, Error)]
pub enum DownloadError {
    #[error(transparent)]
    Provider(#[from] ProviderError),

    #[error(transparent)]
    Library(#[from] LibraryError),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error(
        "Cannot download local track '{name}' (id: {id}): track is already local or has no remote provider"
    )]
    LocalTrack { name: String, id: i64 },

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for DownloadError {
    fn into_response(self) -> Response {
        ApiError::from(self).into_response()
    }
}

impl DownloadError {
    /// Formats a sanitized user-facing error message with long URLs truncated.
    pub fn user_message(&self) -> String {
        let mut msg = self.to_string();
        if let Some(idx) = msg.find("url (") {
            msg.truncate(idx + 4);
            msg.push_str("...)");
        } else if let Some(idx) = msg.find("for url") {
            msg.truncate(idx + 7);
            msg.push_str(" (...)");
        }
        msg
    }
}

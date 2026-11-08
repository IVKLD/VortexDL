use std::io;

use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::{api::errors::ApiError, infra::AudioVerifyError, settings::SettingsError};

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("SoundCloud error: {0}")]
    SoundCloud(#[from] soundcloud_rs::Error),

    #[error("YouTube error: {0}")]
    YouTube(#[from] yt_audio_downloader::YoutubeAudioError),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Invalid URL: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("Upstream returned HTTP {0}")]
    UpstreamStatus(u16),

    #[error("Unsupported music URL: '{0}'. Supported platforms: SoundCloud and YouTube.")]
    UnsupportedPlatform(String),

    #[error("HTTP error: {0}")]
    Http(#[from] axum::http::Error),

    #[error("{0}")]
    InvalidStream(String),

    #[error(transparent)]
    AudioVerify(#[from] AudioVerifyError),

    #[error(transparent)]
    Settings(#[from] SettingsError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl ProviderError {
    pub fn is_expired_status(&self) -> bool {
        matches!(self, Self::UpstreamStatus(401 | 403 | 404 | 410))
    }
}

impl IntoResponse for ProviderError {
    fn into_response(self) -> Response {
        ApiError::from(self).into_response()
    }
}

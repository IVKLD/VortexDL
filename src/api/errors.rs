use std::{convert::Infallible, fmt, io};

use anyhow::Error as AnyhowError;
use axum::{
    Json,
    http::{Error as HttpError, StatusCode},
    response::{IntoResponse, Response},
};
use reqwest::Error as ReqwestError;
use serde::{Deserialize, Serialize};
use serde_json::json;
use soundcloud_rs::Error as SoundCloudError;
use yt_audio_downloader::YoutubeAudioError;

use crate::{
    backup::BackupError, devices::DeviceError, downloader::DownloadError, providers::ProviderError,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    InternalError,
    DatabaseError,
    IoError,
    DeviceError,
    NetworkError,
    SoundCloudError,

    BadRequest,
    EmptyUrl,

    NotFound,
    TrackNotFound,

    Conflict,
    AlreadyProcessing,
}

#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub code: ErrorCode,
    pub message: String,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ApiError {}

impl ApiError {
    pub fn new(status: StatusCode, code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
        }
    }

    pub fn with_code(mut self, code: ErrorCode) -> Self {
        self.code = code;
        self
    }

    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, ErrorCode::BadRequest, msg)
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::InternalError,
            msg,
        )
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, ErrorCode::NotFound, msg)
    }

    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, ErrorCode::Conflict, msg)
    }

    pub fn resolve_reqwest(err: ReqwestError) -> Self {
        let status = if err.is_timeout() {
            StatusCode::GATEWAY_TIMEOUT
        } else if err.is_connect() {
            StatusCode::BAD_GATEWAY
        } else {
            err.status().unwrap_or(StatusCode::BAD_GATEWAY)
        };
        Self::new(status, ErrorCode::NetworkError, err.to_string())
    }

    pub fn resolve_provider(err: ProviderError) -> Self {
        match err {
            ProviderError::SoundCloud(e) => Self::from(e),
            ProviderError::YouTube(e) => Self::from(e),
            ProviderError::Network(e) => Self::resolve_reqwest(e),
            ProviderError::Io(e) => Self::from(e),
            ProviderError::UrlParse(e) => Self::bad_request(e.to_string()),
            ProviderError::UpstreamStatus(code) => Self::new(
                StatusCode::from_u16(code).unwrap_or(StatusCode::BAD_GATEWAY),
                ErrorCode::NetworkError,
                format!("Upstream returned HTTP {code}"),
            ),
            ProviderError::UnsupportedPlatform(url) => Self::bad_request(format!(
                "Unsupported music URL: '{url}'. Supported platforms: SoundCloud and YouTube."
            )),
            ProviderError::InvalidStream(msg) => {
                Self::new(StatusCode::BAD_GATEWAY, ErrorCode::NetworkError, msg)
            }
            ProviderError::Other(e) => Self::from(e),
        }
    }

    pub fn resolve_download(err: DownloadError) -> Self {
        match err {
            DownloadError::Provider(e) => Self::resolve_provider(e),
            DownloadError::Io(e) => Self::from(e),
            DownloadError::LocalTrack { .. } => Self::bad_request(err.to_string()),
            DownloadError::Other(e) => Self::from(e),
        }
    }

    pub fn resolve_device(err: DeviceError) -> Self {
        match err {
            DeviceError::AlreadyInProgress => Self::new(
                StatusCode::CONFLICT,
                ErrorCode::AlreadyProcessing,
                "Device is currently syncing",
            ),
            DeviceError::NotAvailable => Self::new(
                StatusCode::SERVICE_UNAVAILABLE,
                ErrorCode::DeviceError,
                "ADB binary not found in PATH",
            ),
            DeviceError::Other(e) => Self::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::DeviceError,
                format!("{e:#}"),
            ),
        }
    }

    pub fn resolve_backup(err: BackupError) -> Self {
        match err {
            BackupError::Auth => Self::new(
                StatusCode::UNAUTHORIZED,
                ErrorCode::InternalError,
                err.to_string(),
            ),
            BackupError::NotFound => Self::not_found(err.to_string()),
            BackupError::HttpStatus(status) => {
                Self::new(status, ErrorCode::NetworkError, err.to_string())
            }
            BackupError::Network(e) => Self::resolve_reqwest(e),
            BackupError::FileSystem(e) => Self::from(e),
            BackupError::Database(e) => Self::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::DatabaseError,
                e.to_string(),
            ),
            BackupError::Serialization(e) => Self::bad_request(e.to_string()),
            BackupError::OperationNotSupported(msg) => {
                Self::new(StatusCode::METHOD_NOT_ALLOWED, ErrorCode::BadRequest, msg)
            }
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = Json(json!({
            "error": self.message,
            "code": self.code
        }));
        (self.status, body).into_response()
    }
}

impl From<AnyhowError> for ApiError {
    fn from(err: AnyhowError) -> Self {
        Self::internal(format!("{err:#}"))
    }
}

impl From<io::Error> for ApiError {
    fn from(err: io::Error) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::IoError,
            err.to_string(),
        )
    }
}

impl From<ReqwestError> for ApiError {
    fn from(err: ReqwestError) -> Self {
        Self::resolve_reqwest(err)
    }
}

impl From<YoutubeAudioError> for ApiError {
    fn from(err: YoutubeAudioError) -> Self {
        Self::new(
            StatusCode::BAD_GATEWAY,
            ErrorCode::NetworkError,
            err.to_string(),
        )
    }
}

impl From<SoundCloudError> for ApiError {
    fn from(err: SoundCloudError) -> Self {
        Self::new(
            StatusCode::BAD_GATEWAY,
            ErrorCode::SoundCloudError,
            err.to_string(),
        )
    }
}

impl From<HttpError> for ApiError {
    fn from(err: HttpError) -> Self {
        Self::internal(err.to_string())
    }
}

impl From<ProviderError> for ApiError {
    fn from(err: ProviderError) -> Self {
        Self::resolve_provider(err)
    }
}

impl From<DownloadError> for ApiError {
    fn from(err: DownloadError) -> Self {
        Self::resolve_download(err)
    }
}

impl From<DeviceError> for ApiError {
    fn from(err: DeviceError) -> Self {
        Self::resolve_device(err)
    }
}

impl From<BackupError> for ApiError {
    fn from(err: BackupError) -> Self {
        Self::resolve_backup(err)
    }
}

impl From<Infallible> for ApiError {
    fn from(err: Infallible) -> Self {
        match err {}
    }
}

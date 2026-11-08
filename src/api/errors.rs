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
    backup::BackupError, devices::DeviceError, downloader::DownloadError, library::LibraryError,
    providers::ProviderError, search::SearchError, settings::SettingsError, stream::StreamError,
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
        let status = if err.is_timeout() {
            StatusCode::GATEWAY_TIMEOUT
        } else if err.is_connect() {
            StatusCode::BAD_GATEWAY
        } else {
            err.status().unwrap_or(StatusCode::BAD_GATEWAY)
        };
        Self::new(status, ErrorCode::NetworkError, err.to_string())
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
        match err {
            ProviderError::SoundCloud(e) => e.into(),
            ProviderError::YouTube(e) => e.into(),
            ProviderError::Network(e) => e.into(),
            ProviderError::Io(e) => e.into(),
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
            ProviderError::AudioVerify(e) => {
                Self::new(StatusCode::BAD_GATEWAY, ErrorCode::NetworkError, e.to_string())
            }
            ProviderError::Http(e) => e.into(),
            ProviderError::Settings(e) => e.into(),
            ProviderError::Other(e) => e.into(),
        }
    }
}

impl From<DownloadError> for ApiError {
    fn from(err: DownloadError) -> Self {
        match err {
            DownloadError::Provider(e) => e.into(),
            DownloadError::Library(e) => e.into(),
            DownloadError::Io(e) => e.into(),
            DownloadError::LocalTrack { .. } => Self::bad_request(err.to_string()),
            DownloadError::Other(e) => e.into(),
        }
    }
}

impl From<DeviceError> for ApiError {
    fn from(err: DeviceError) -> Self {
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
}

impl From<BackupError> for ApiError {
    fn from(err: BackupError) -> Self {
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
            BackupError::Network(e) => e.into(),
            BackupError::FileSystem(e) => e.into(),
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

impl From<StreamError> for ApiError {
    fn from(err: StreamError) -> Self {
        match err {
            StreamError::TrackNotFound(id) => {
                Self::not_found(format!("Track {id} not found")).with_code(ErrorCode::TrackNotFound)
            }
            StreamError::UnsupportedPlatform(id) => Self::bad_request(format!(
                "Unsupported or local music platform for track ID: {id}"
            )),
            StreamError::Provider(e) => e.into(),
            StreamError::Http(e) => e.into(),
            StreamError::Io(e) => e.into(),
        }
    }
}

impl From<SearchError> for ApiError {
    fn from(err: SearchError) -> Self {
        match err {
            SearchError::EmptyQuery => Self::bad_request("Search query cannot be empty"),
            SearchError::SoundCloud(e) => e.into(),
            SearchError::Provider(e) => e.into(),
        }
    }
}

impl From<LibraryError> for ApiError {
    fn from(err: LibraryError) -> Self {
        match err {
            LibraryError::Io(e) => e.into(),
            LibraryError::Metadata(e) => Self::internal(e.to_string()),
            LibraryError::Id3(e) => Self::internal(e.to_string()),
            LibraryError::Database(e) => {
                Self::new(StatusCode::INTERNAL_SERVER_ERROR, ErrorCode::DatabaseError, e.to_string())
            }
            LibraryError::TaskJoin(e) => Self::internal(e.to_string()),
            LibraryError::TrackNotFound(id) => {
                Self::not_found(format!("Track {id} not found")).with_code(ErrorCode::TrackNotFound)
            }
            LibraryError::Other(e) => Self::internal(format!("{e:#}")),
        }
    }
}

impl From<SettingsError> for ApiError {
    fn from(err: SettingsError) -> Self {
        match err {
            SettingsError::Serialization(e) => Self::bad_request(e.to_string()),
            SettingsError::Database(e) => {
                Self::new(StatusCode::INTERNAL_SERVER_ERROR, ErrorCode::DatabaseError, e.to_string())
            }
            SettingsError::Other(e) => Self::internal(format!("{e:#}")),
        }
    }
}

impl From<Infallible> for ApiError {
    fn from(err: Infallible) -> Self {
        match err {}
    }
}

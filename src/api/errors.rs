use std::{convert::Infallible, io};

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

use crate::{adb::AdbError, backup::BackupError};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    InternalError,
    DatabaseError,
    IoError,
    AdbError,
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
        Self::internal(format!("{:#}", err))
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
        } else if let Some(s) = err.status() {
            s
        } else {
            StatusCode::BAD_GATEWAY
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

impl From<AdbError> for ApiError {
    fn from(err: AdbError) -> Self {
        match err {
            AdbError::AlreadyInProgress => Self::new(
                StatusCode::CONFLICT,
                ErrorCode::AlreadyProcessing,
                "Device is currently syncing",
            ),
            AdbError::NotAvailable => Self::new(
                StatusCode::SERVICE_UNAVAILABLE,
                ErrorCode::AdbError,
                "ADB binary not found in PATH",
            ),
            AdbError::Other(e) => Self::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::AdbError,
                format!("{:#}", e),
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
            BackupError::NotFound => {
                Self::new(StatusCode::NOT_FOUND, ErrorCode::NotFound, err.to_string())
            }
            BackupError::HttpStatus(status) => {
                Self::new(status, ErrorCode::NetworkError, err.to_string())
            }
            BackupError::Network(e) => Self::new(
                StatusCode::BAD_GATEWAY,
                ErrorCode::NetworkError,
                e.to_string(),
            ),
            BackupError::FileSystem(e) => Self::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::IoError,
                e.to_string(),
            ),
            BackupError::Database(e) => Self::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::DatabaseError,
                e.to_string(),
            ),
            BackupError::Serialization(e) => Self::new(
                StatusCode::BAD_REQUEST,
                ErrorCode::BadRequest,
                e.to_string(),
            ),
            BackupError::OperationNotSupported(e) => {
                Self::new(StatusCode::METHOD_NOT_ALLOWED, ErrorCode::BadRequest, e)
            }
        }
    }
}

impl From<Infallible> for ApiError {
    fn from(err: Infallible) -> Self {
        match err {}
    }
}

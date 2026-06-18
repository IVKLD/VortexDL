use std::io;

use anyhow::Error as AnyhowError;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    InternalError,
    DatabaseError,
    IoError,
    AdbError,
    NetworkError,
    HttpClientError,
    SoundCloudError,
    
    BadRequest,
    InvalidProxyUrl,
    EmptyUrl,
    
    NotFound,
    TrackNotFound,
    FileNotFound,
    DeviceNotFound,
    
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
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, ErrorCode::InternalError, msg)
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
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, ErrorCode::IoError, err.to_string())
    }
}

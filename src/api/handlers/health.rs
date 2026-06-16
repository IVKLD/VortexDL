use axum::{Json, http::StatusCode, response::IntoResponse};

use crate::types::api::{ApiStatus, HealthResponse};

pub async fn health() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(HealthResponse {
            status: ApiStatus::Ok,
        }),
    )
}

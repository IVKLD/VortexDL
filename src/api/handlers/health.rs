use axum::{Json, http::StatusCode, response::IntoResponse};

use crate::api::models::HealthResponse;

pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(HealthResponse { status: "ok" }))
}

use axum::{Json, response::IntoResponse, http::StatusCode};
use crate::api::models::HealthResponse;

pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(HealthResponse { status: "ok" }))
}

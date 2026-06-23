use axum::{Json, http::StatusCode, response::IntoResponse};

use crate::api::types::{ApiStatus, HealthResponse};

#[utoipa::path(
    method(get),
    path = "/api/health",
    responses(
        (status = 200, description = "Get API health status", body = HealthResponse)
    )
)]
pub async fn health() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(HealthResponse {
            status: ApiStatus::Ok,
        }),
    )
}

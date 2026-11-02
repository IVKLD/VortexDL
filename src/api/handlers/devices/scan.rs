use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::{api::errors::ApiError, app::AppState};

#[utoipa::path(
    method(post),
    path = "/api/devices/{device_id}/scan",
    params(
        ("device_id" = String, Path, description = "ADB Device ID")
    ),
    responses(
        (status = 200, description = "Player indexing scan triggered successfully")
    )
)]
pub async fn scan_adb_device(
    State(state): State<AppState>,
    Path(device_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    state.device_service().scan_device(&device_id).await?;
    Ok(StatusCode::OK)
}

use axum::{extract::Path, http::StatusCode, response::IntoResponse};

use crate::{adb::scan_device, api::errors::ApiError};

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
pub async fn scan_adb_device(Path(device_id): Path<String>) -> Result<impl IntoResponse, ApiError> {
    scan_device(&device_id).await?;
    Ok(StatusCode::OK)
}

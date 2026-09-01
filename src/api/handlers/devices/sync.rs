use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::{
    adb::sync_device,
    api::{errors::ApiError, state::AppState},
};

#[utoipa::path(
    method(post),
    path = "/api/devices/{device_id}/sync",
    params(
        ("device_id" = String, Path, description = "ADB Device ID")
    ),
    responses(
        (status = 200, description = "Sync completed successfully")
    )
)]
pub async fn sync_adb_device(
    Path(device_id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let adb_settings = state.settings.read().await.adb.clone();
    let device_cfg = adb_settings
        .devices
        .iter()
        .find(|d| d.device_id == device_id);

    let remote_music_dir = match device_cfg {
        Some(cfg) if !cfg.remote_music_dir.is_empty() => cfg.remote_music_dir.clone(),
        _ => {
            return Err(ApiError::bad_request(
                "Device configuration or remote music directory is not set",
            ));
        }
    };

    sync_device(&device_id, &remote_music_dir, state.storage.clone(), true).await?;
    Ok(StatusCode::OK)
}

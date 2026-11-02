use axum::{
    Json,
    extract::{Path, State},
};

use crate::{api::errors::ApiError, app::AppState, devices::StorageInfo};

#[utoipa::path(
    method(get),
    path = "/api/devices",
    responses(
        (status = 200, description = "List connected ADB devices", body = Vec<String>)
    )
)]
pub async fn list_adb_devices(
    State(state): State<AppState>,
) -> Result<Json<Vec<String>>, ApiError> {
    let devices = state.device_service().list_devices().await?;
    let mut list: Vec<String> = devices.into_iter().collect();
    list.sort();
    Ok(Json(list))
}

#[utoipa::path(
    method(get),
    path = "/api/devices/{device_id}/storage",
    params(
        ("device_id" = String, Path, description = "ADB Device ID")
    ),
    responses(
        (status = 200, description = "Get device storage partitions", body = Vec<StorageInfo>)
    )
)]
pub async fn get_device_storage_info(
    State(state): State<AppState>,
    Path(device_id): Path<String>,
) -> Result<Json<Vec<StorageInfo>>, ApiError> {
    let storages = state
        .device_service()
        .get_device_storages(&device_id)
        .await?;
    Ok(Json(storages))
}

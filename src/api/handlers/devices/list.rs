use axum::{Json, extract::Path};

use crate::{
    adb::{StorageInfo, get_device_storages, list_devices},
    api::errors::{ApiError, ErrorCode},
};

#[utoipa::path(
    method(get),
    path = "/api/devices",
    responses(
        (status = 200, description = "List connected ADB devices", body = Vec<String>)
    )
)]
pub async fn list_adb_devices() -> Result<Json<Vec<String>>, ApiError> {
    let devices = list_devices().await.map_err(|e| {
        ApiError::internal(format!("Failed to list ADB devices: {e}"))
            .with_code(ErrorCode::AdbError)
    })?;

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
    Path(device_id): Path<String>,
) -> Result<Json<Vec<StorageInfo>>, ApiError> {
    let storages = get_device_storages(&device_id).await.map_err(|e| {
        ApiError::internal(format!(
            "Failed to list storage partitions for device {device_id}: {e}"
        ))
        .with_code(ErrorCode::AdbError)
    })?;

    Ok(Json(storages))
}

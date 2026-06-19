use axum::Json;

use crate::{
    adb_device::list_connected_devices,
    api::errors::{ApiError, ErrorCode},
};

pub async fn list_adb_devices() -> Result<Json<Vec<String>>, ApiError> {
    let devices = list_connected_devices().await.map_err(|e| {
        ApiError::internal(format!("Failed to list ADB devices: {e}"))
            .with_code(ErrorCode::AdbError)
    })?;

    let mut list: Vec<String> = devices.into_iter().collect();
    list.sort();

    Ok(Json(list))
}

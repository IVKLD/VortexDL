use axum::Json;

use crate::{api::errors::ApiError, device::adb::discovery::list_connected_devices};

pub async fn list_adb_devices() -> Result<Json<Vec<String>>, ApiError> {
    let devices = list_connected_devices()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to list ADB devices: {e}")))?;

    let mut list: Vec<String> = devices.into_iter().collect();
    list.sort();

    Ok(Json(list))
}

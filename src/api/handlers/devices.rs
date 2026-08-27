use axum::{
    Json,
    extract::{
        Path, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::StatusCode,
    response::IntoResponse,
};

use crate::{
    adb::{AdbError, StorageInfo, get_device_storages, list_devices, sync_device},
    api::{
        errors::{ApiError, ErrorCode},
        state::AppState,
    },
};

#[utoipa::path(
    method(get),
    path = "/api/devices/ws",
    responses(
        (status = 101, description = "WebSocket upgrade to stream connected ADB devices")
    )
)]
pub async fn devices_ws(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    let mut last_devices = Vec::new();
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(2));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                if let Ok(devices) = list_devices().await {
                    let mut list: Vec<String> = devices.into_iter().collect();
                    list.sort();

                    if list != last_devices {
                        last_devices = list.clone();
                        let msg_str = match serde_json::to_string(&list) {
                            Ok(s) => s,
                            Err(_) => continue,
                        };
                        if socket.send(Message::Text(msg_str.into())).await.is_err() {
                            break;
                        }
                    }
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(_)) => {}
                    _ => break,
                }
            }
        }
    }
}

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

    sync_device(&device_id, &remote_music_dir, state.storage.clone(), true)
        .await
        .map_err(|e| match e.downcast_ref::<AdbError>() {
            Some(AdbError::AlreadyInProgress) => ApiError::new(
                StatusCode::CONFLICT,
                ErrorCode::AlreadyProcessing,
                "Device is currently syncing",
            ),
            _ => ApiError::internal(format!("Failed to sync device {device_id}: {e}"))
                .with_code(ErrorCode::AdbError),
        })?;

    Ok(StatusCode::OK)
}

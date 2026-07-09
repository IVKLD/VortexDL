pub mod commands;
pub mod state;
pub mod sync;
pub mod ui;

use std::{collections::HashSet, sync::Arc, time::Duration};

pub use commands::{AdbError, StorageInfo, StorageType, get_device_storages, list_devices};
pub use sync::sync_device;
use tokio::sync::RwLock;

use crate::{settings::SettingsManager, storage::MusicStorage};

pub fn init(storage: Arc<RwLock<MusicStorage>>, settings: SettingsManager) {
    tokio::spawn(async move {
        let mut tracker_task: Option<tokio::task::JoinHandle<Result<(), AdbError>>> = None;
        let mut adb_available = true;

        loop {
            let adb_enabled = settings.read().await.adb.enabled;

            if adb_enabled && adb_available {
                let is_finished = tracker_task
                    .as_ref()
                    .map(|h| h.is_finished())
                    .unwrap_or(true);
                if is_finished {
                    if let Some(h) = tracker_task.take() {
                        match h.await {
                            Ok(Ok(())) => {}
                            Ok(Err(AdbError::NotAvailable)) => {
                                tracing::warn!(
                                    "adb binary not found in PATH — ADB device tracking disabled"
                                );
                                adb_available = false;
                            }
                            Ok(Err(e)) => {
                                tracing::warn!(error = %e, "ADB device tracker failed, will retry");
                            }
                            Err(join_err) => {
                                if !join_err.is_cancelled() {
                                    tracing::warn!("ADB device tracker task panicked: {join_err}");
                                }
                            }
                        }
                    }
                    if adb_available {
                        let storage = storage.clone();
                        let settings = settings.clone();
                        tracker_task = Some(tokio::spawn(async move {
                            run_device_tracker(storage, settings).await
                        }));
                    }
                }
            } else if let Some(handle) = tracker_task.take() {
                tracing::info!("ADB disabled, stopping tracker task");
                handle.abort();
                state::lock_connected().clear();
            }

            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    });
}

async fn ensure_adb_server_running() -> Result<(), AdbError> {
    let output = commands::run_adb_raw(&["start-server"]).await?;

    if !output.status.success() {
        return Err(AdbError::Other(anyhow::anyhow!(
            "adb start-server failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(())
}

async fn connect_and_subscribe() -> Result<tokio::net::TcpStream, AdbError> {
    let host = std::env::var("ADB_SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("ADB_SERVER_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(5037);
    let addr = format!("{host}:{port}");

    let mut stream = match tokio::net::TcpStream::connect(&addr).await {
        Ok(s) => s,
        Err(_) => {
            ensure_adb_server_running().await?;
            tokio::net::TcpStream::connect(&addr).await.map_err(|e| {
                AdbError::Other(anyhow::anyhow!(
                    "Failed to connect to ADB server at {addr} after start-server: {e}"
                ))
            })?
        }
    };

    use tokio::io::AsyncWriteExt;
    if let Err(e) = stream.write_all(b"0012host:track-devices").await {
        return Err(AdbError::Other(anyhow::anyhow!(
            "Failed to write track-devices request: {e}"
        )));
    }

    // Read 4-byte response status
    use tokio::io::AsyncReadExt;
    let mut status = [0u8; 4];
    if let Err(e) = stream.read_exact(&mut status).await {
        return Err(AdbError::Other(anyhow::anyhow!(
            "Failed to read track-devices status: {e}"
        )));
    }

    if &status == b"OKAY" {
        Ok(stream)
    } else if &status == b"FAIL" {
        let payload = read_adb_hex_packet(&mut stream).await?;
        let err_msg = String::from_utf8_lossy(&payload).into_owned();
        Err(AdbError::Other(anyhow::anyhow!(
            "ADB server returned FAIL: {err_msg}"
        )))
    } else {
        Err(AdbError::Other(anyhow::anyhow!(
            "ADB server returned unexpected status: {:?}",
            status
        )))
    }
}

async fn read_adb_hex_packet(stream: &mut tokio::net::TcpStream) -> Result<Vec<u8>, AdbError> {
    use tokio::io::AsyncReadExt;
    let mut len_buf = [0u8; 4];
    stream
        .read_exact(&mut len_buf)
        .await
        .map_err(|e| AdbError::Other(anyhow::anyhow!("Failed to read packet length: {e}")))?;
    let len_str = std::str::from_utf8(&len_buf)
        .map_err(|e| AdbError::Other(anyhow::anyhow!("Length not valid UTF-8: {e}")))?;
    let len = usize::from_str_radix(len_str, 16)
        .map_err(|e| AdbError::Other(anyhow::anyhow!("Length not valid hex '{len_str}': {e}")))?;

    let mut payload = vec![0u8; len];
    stream
        .read_exact(&mut payload)
        .await
        .map_err(|e| AdbError::Other(anyhow::anyhow!("Failed to read packet payload: {e}")))?;
    Ok(payload)
}

async fn read_device_list_update(
    stream: &mut tokio::net::TcpStream,
) -> Result<HashSet<String>, AdbError> {
    let payload = read_adb_hex_packet(stream).await?;
    let payload_str = String::from_utf8(payload)
        .map_err(|e| AdbError::Other(anyhow::anyhow!("Payload not valid UTF-8: {e}")))?;

    let devices = payload_str
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let id = parts.next()?;
            (parts.next()? == "device").then(|| id.to_string())
        })
        .collect();

    Ok(devices)
}

async fn run_device_tracker(
    storage: Arc<RwLock<MusicStorage>>,
    settings: SettingsManager,
) -> Result<(), AdbError> {
    let mut stream = connect_and_subscribe().await?;
    tracing::info!("Successfully connected to ADB server and subscribed to device tracking");

    loop {
        let current = read_device_list_update(&mut stream).await?;

        let adb_settings = {
            let s = settings.read().await;
            s.adb.clone()
        };

        let mut previous = state::lock_connected();

        for id in current.difference(&previous) {
            tracing::info!(device = %id, "connected");
            if let Some(cfg) = adb_settings
                .devices
                .iter()
                .find(|d| d.enabled && d.device_id == *id)
            {
                spawn_sync(id.clone(), cfg.remote_music_dir.clone(), storage.clone());
            }
        }

        for id in previous.difference(&current) {
            tracing::info!(device = %id, "disconnected");
        }

        *previous = current;
    }
}

pub async fn sync_connected(storage: Arc<RwLock<MusicStorage>>, settings: SettingsManager) {
    let s = settings.read().await;
    if !s.adb.enabled || !s.adb.auto_sync {
        return;
    }
    let devices = s.adb.devices.clone();
    drop(s);

    let connected = state::lock_connected();
    for id in connected.iter() {
        if let Some(cfg) = devices.iter().find(|d| d.enabled && d.device_id == *id) {
            spawn_sync(id.clone(), cfg.remote_music_dir.clone(), storage.clone());
        }
    }
}

fn spawn_sync(device_id: String, remote_music_dir: String, storage: Arc<RwLock<MusicStorage>>) {
    tokio::spawn(async move {
        if let Err(e) = sync_device(&device_id, &remote_music_dir, storage).await {
            tracing::error!(device = %device_id, error = %e, "sync failed");
        }
    });
}

#[cfg(test)]
mod tests {
    use tokio::{io::AsyncWriteExt, net::TcpListener};

    use super::*;

    #[tokio::test]
    async fn test_read_device_list_update() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server_task = tokio::spawn(async move {
            use tokio::io::AsyncReadExt;
            let (mut socket, _) = listener.accept().await.unwrap();
            socket.write_all(b"0011device123\tdevice\n").await.unwrap();
            socket.write_all(b"0000").await.unwrap();
            let mut buf = [0u8; 1];
            let _ = socket.read(&mut buf).await;
        });

        let mut client = tokio::net::TcpStream::connect(addr).await.unwrap();

        let devices = read_device_list_update(&mut client).await.unwrap();
        assert_eq!(devices.len(), 1);
        assert!(devices.contains("device123"));

        let devices = read_device_list_update(&mut client).await.unwrap();
        assert!(devices.is_empty());

        drop(client);

        server_task.await.unwrap();
    }
}

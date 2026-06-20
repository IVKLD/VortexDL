pub mod commands;
pub mod state;
pub mod sync;
pub mod ui;

use std::{sync::Arc, time::Duration};

pub use commands::{AdbError, StorageInfo, get_device_storages, list_devices};
pub use sync::sync_device;
use tokio::sync::RwLock;

use crate::{settings::SettingsManager, storage::MusicStorage};

pub fn init(storage: Arc<RwLock<MusicStorage>>, settings: SettingsManager) {
    tokio::spawn(async move {
        loop {
            match poll_devices(storage.clone(), settings.clone()).await {
                Ok(()) => {}
                Err(AdbError::NotAvailable) => {
                    tracing::warn!("adb binary not found in PATH — ADB polling disabled");
                    break;
                }
                Err(e) => {
                    tracing::warn!(error = %e, "ADB poll failed");
                }
            }
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    });
}

async fn poll_devices(storage: Arc<RwLock<MusicStorage>>, settings: SettingsManager) -> Result<(), AdbError> {
    let settings_read = settings.read().await;
    if !settings_read.adb.enabled {
        return Ok(());
    }

    let current = list_devices().await?;
    let mut previous = state::lock_connected();

    for id in current.difference(&previous) {
        tracing::info!(device = %id, "connected");
        if let Some(cfg) = settings_read
            .adb
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
    Ok(())
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

pub mod commands;
pub mod state;
pub mod sync;
pub mod ui;

use std::{collections::HashSet, sync::Arc, time::Duration};

use anyhow::Result;
pub use commands::list_connected_devices;
pub use sync::sync_device;
use tokio::sync::RwLock;

use crate::{
    settings::{AdbDeviceSettings, SettingsManager},
    storage::MusicStorage,
};

pub fn init(storage: Arc<RwLock<MusicStorage>>, settings: SettingsManager) {
    tokio::spawn(async move {
        loop {
            if let Err(e) = check_devices(storage.clone(), settings.clone()).await {
                tracing::error!(error = %e, "ADB device check failed");
            }
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    });
}

pub async fn check_devices(
    storage: Arc<RwLock<MusicStorage>>,
    settings: SettingsManager,
) -> Result<()> {
    let settings_read = settings.read().await;
    if !settings_read.adb.enabled {
        return Ok(());
    }
    drop(settings_read);

    let current = list_connected_devices().await?;

    let mut state = state::CONNECTED_DEVICES.lock().await;
    let previous = state.get_or_insert_with(HashSet::new);

    for id in current.difference(previous) {
        tracing::info!(device = %id, "ADB device connected");
        if let Some(cfg) = find_device_config(&settings, id).await {
            spawn_sync(id.clone(), cfg, storage.clone());
        } else {
            tracing::debug!(device = %id, "Not in allowed devices list, skipping sync");
        }
    }

    for id in previous.difference(&current) {
        tracing::info!(device = %id, "ADB device disconnected");
    }

    *previous = current;
    Ok(())
}

pub async fn sync_all_connected(storage: Arc<RwLock<MusicStorage>>, settings: SettingsManager) {
    let settings_read = settings.read().await;
    if !settings_read.adb.enabled || !settings_read.adb.auto_sync {
        return;
    }
    drop(settings_read);

    let state = state::CONNECTED_DEVICES.lock().await;
    let Some(devices) = state.as_ref() else {
        return;
    };

    if devices.is_empty() {
        return;
    }

    for device_id in devices {
        if let Some(cfg) = find_device_config(&settings, device_id).await {
            spawn_sync(device_id.clone(), cfg, storage.clone());
        }
    }
}

async fn find_device_config(
    settings: &SettingsManager,
    device_id: &str,
) -> Option<AdbDeviceSettings> {
    settings
        .read()
        .await
        .adb
        .devices
        .iter()
        .find(|d| d.enabled && d.device_id == device_id)
        .cloned()
}

fn spawn_sync(device_id: String, cfg: AdbDeviceSettings, storage: Arc<RwLock<MusicStorage>>) {
    tokio::spawn(async move {
        if let Err(e) = sync_device(&device_id, &cfg.remote_music_dir, storage).await {
            tracing::error!(device = %device_id, error = %e, "Sync failed");
        }
    });
}

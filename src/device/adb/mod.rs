use std::{collections::HashSet, sync::Arc};

use anyhow::Result;
use tokio::sync::{Mutex, RwLock};

use crate::{
    database::settings::{AdbDeviceSettings, UserSettings},
    storage::MusicStorage,
};

pub mod commands;
pub mod discovery;
pub mod sync;
pub mod ui;

static CONNECTED_DEVICES: Mutex<Option<HashSet<String>>> = Mutex::const_new(None);

pub async fn check_devices(
    storage: Arc<RwLock<MusicStorage>>,
    settings: Arc<RwLock<UserSettings>>,
) -> Result<()> {
    let settings_read = settings.read().await;
    if !settings_read.adb.enabled {
        return Ok(());
    }
    drop(settings_read);

    let current = discovery::list_connected_devices().await?;

    let mut state = CONNECTED_DEVICES.lock().await;
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

pub async fn sync_all_connected(
    storage: Arc<RwLock<MusicStorage>>,
    settings: Arc<RwLock<UserSettings>>,
) {
    let settings_read = settings.read().await;
    if !settings_read.adb.enabled || !settings_read.adb.auto_sync {
        return;
    }
    drop(settings_read);

    let state = CONNECTED_DEVICES.lock().await;
    let Some(devices) = state.as_ref() else {
        return;
    };

    if devices.is_empty() {
        return;
    }

    for device_id in devices {
        if let Some(cfg) = find_device_config(&settings, device_id).await {
            tracing::info!(device = %device_id, "Triggering post-download sync");
            spawn_sync(device_id.clone(), cfg, storage.clone());
        }
    }
}

async fn find_device_config(
    settings: &Arc<RwLock<UserSettings>>,
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
        if let Err(e) = sync::sync_device(&device_id, &cfg.remote_music_dir, storage).await {
            tracing::error!(device = %device_id, error = %e, "Sync failed");
        }
    });
}

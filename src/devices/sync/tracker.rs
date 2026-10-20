use std::{sync::Arc, time::Duration};

use tokio::sync::RwLock;

use super::engine::sync_device;
use crate::{
    devices::{
        adb::{read_device_list_update, subscribe_track_devices},
        error::DeviceError,
        registry,
    },
    library::Library,
    settings::SettingsManager,
};

/// What: Spawns the ADB supervisor task that manages device tracking lifecycle.
/// Flow: Called during application initialization in `src/main.rs`.
/// Why:  Periodically checks if ADB is enabled in user settings. If enabled, starts `run_device_tracker`.
///       If disabled, aborts the tracker task and clears connected state.
pub fn init_device_tracker(storage: Arc<RwLock<Library>>, settings: SettingsManager) {
    tokio::spawn(async move {
        let mut tracker_task: Option<tokio::task::JoinHandle<Result<(), DeviceError>>> = None;
        let mut adb_available = true;
        let mut settings_rx = settings.subscribe_changes();

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
                            Ok(Err(DeviceError::NotAvailable)) => {
                                tracing::warn!(
                                    "ADB binary not found, disabling automatic device tracking until settings update"
                                );
                                adb_available = false;
                            }
                            Ok(Err(e)) => {
                                tracing::warn!("ADB tracker task ended with error: {e}");
                            }
                            Err(e) => {
                                tracing::warn!("ADB tracker task panicked: {e}");
                            }
                        }
                    }

                    if adb_available {
                        let s = storage.clone();
                        let cfg = settings.clone();
                        tracker_task =
                            Some(tokio::spawn(
                                async move { run_device_tracker(s, cfg).await },
                            ));
                    }
                }
            } else if let Some(handle) = tracker_task.take() {
                tracing::info!("ADB disabled, stopping tracker task");
                handle.abort();
                registry::lock_connected().clear();
            }

            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(3)) => {}
                _ = settings_rx.changed() => {
                    adb_available = true;
                }
            }
        }
    });
}

/// What: Background listener loop that monitors ADB device connection and disconnection events.
/// Flow: Spawned by `init`; runs continuously while ADB is enabled in settings.
/// Why:  Subscribes directly to the ADB daemon socket (`host:track-devices`). When an authorized
///       device connects, it automatically triggers `sync_device` without requiring user interaction.
async fn run_device_tracker(
    storage: Arc<RwLock<Library>>,
    settings: SettingsManager,
) -> Result<(), DeviceError> {
    let mut stream = subscribe_track_devices().await?;
    tracing::info!("Successfully connected to ADB server and subscribed to device tracking");

    loop {
        let current = read_device_list_update(&mut stream).await?;

        let adb_settings = {
            let s = settings.read().await;
            s.adb.clone()
        };

        let mut previous = registry::lock_connected();

        for id in current.difference(&previous) {
            tracing::info!("ADB device connected: {id}");
            if let Some(cfg) = adb_settings.find_enabled_device(id) {
                spawn_sync(id.clone(), cfg.remote_music_dir.clone(), storage.clone());
            }
        }

        for id in previous.difference(&current) {
            tracing::info!("ADB device disconnected: {id}");
        }

        *previous = current;
    }
}

pub async fn sync_connected(storage: Arc<RwLock<Library>>, settings: SettingsManager) {
    let s = settings.read().await;
    if !s.adb.enabled || !s.adb.auto_sync {
        return;
    }
    let adb_settings = s.adb.clone();
    drop(s);

    let connected = registry::lock_connected();
    for id in connected.iter() {
        if let Some(cfg) = adb_settings.find_enabled_device(id) {
            spawn_sync(
                id.to_string(),
                cfg.remote_music_dir.clone(),
                storage.clone(),
            );
        }
    }
}

fn spawn_sync(device_id: String, remote_music_dir: String, storage: Arc<RwLock<Library>>) {
    tokio::spawn(async move {
        if let Err(e) = sync_device(&device_id, &remote_music_dir, storage, false).await {
            tracing::error!("ADB device {device_id} sync failed: {e}");
        }
    });
}

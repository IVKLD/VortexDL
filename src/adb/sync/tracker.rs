use std::{sync::Arc, time::Duration};

use tokio::sync::RwLock;

use super::engine::sync_device;
use crate::{
    adb::{
        error::AdbError,
        state,
        transport::{read_device_list_update, subscribe_track_devices},
    },
    settings::SettingsManager,
    storage::MusicStorage,
};

/// What: Spawns the ADB supervisor task that manages device tracking lifecycle.
/// Flow: Called during application initialization in `src/main.rs`.
/// Why:  Periodically checks if ADB is enabled in user settings. If enabled, starts `run_device_tracker`.
///       If disabled, aborts the tracker task and clears connected state.
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
                                    "ADB binary not found, disabling automatic device tracking until restart or settings update"
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
                state::lock_connected().clear();
            }

            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    });
}

/// What: Background listener loop that monitors ADB device connection and disconnection events.
/// Flow: Spawned by `init`; runs continuously while ADB is enabled in settings.
/// Why:  Subscribes directly to the ADB daemon socket (`host:track-devices`). When an authorized
///       device connects, it automatically triggers `sync_device` without requiring user interaction.
async fn run_device_tracker(
    storage: Arc<RwLock<MusicStorage>>,
    settings: SettingsManager,
) -> Result<(), AdbError> {
    let mut stream = subscribe_track_devices().await?;
    tracing::info!("Successfully connected to ADB server and subscribed to device tracking");

    loop {
        let current = read_device_list_update(&mut stream).await?;

        let adb_settings = {
            let s = settings.read().await;
            s.adb.clone()
        };

        let mut previous = state::lock_connected();

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

pub async fn sync_connected(storage: Arc<RwLock<MusicStorage>>, settings: SettingsManager) {
    let s = settings.read().await;
    if !s.adb.enabled || !s.adb.auto_sync {
        return;
    }
    let adb_settings = s.adb.clone();
    drop(s);

    let connected = state::lock_connected();
    for id in connected.iter() {
        if let Some(cfg) = adb_settings.find_enabled_device(id) {
            spawn_sync(id.clone(), cfg.remote_music_dir.clone(), storage.clone());
        }
    }
}

fn spawn_sync(device_id: String, remote_music_dir: String, storage: Arc<RwLock<MusicStorage>>) {
    tokio::spawn(async move {
        if let Err(e) = sync_device(&device_id, &remote_music_dir, storage, false).await {
            tracing::error!("ADB device {device_id} sync failed: {e}");
        }
    });
}

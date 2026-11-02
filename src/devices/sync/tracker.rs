use std::time::Duration;

use tokio::task::JoinHandle;

use crate::{
    devices::{adb, error::DeviceError, registry, sync::engine},
    library::LibraryService,
    settings::SettingsManager,
};

/// What: Spawns the ADB supervisor task that manages device tracking lifecycle.
/// Flow: Called during application initialization in `src/main.rs`.
/// Why:  Periodically checks if ADB is enabled in user settings. If enabled, starts `run_device_tracker`.
///       If disabled, aborts the tracker task and clears connected state.
pub fn init_device_tracker(library: LibraryService, settings: SettingsManager) {
    tokio::spawn(async move {
        let mut tracker_task: Option<JoinHandle<Result<(), DeviceError>>> = None;
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
                        let lib = library.clone();
                        let cfg = settings.clone();
                        tracker_task =
                            Some(tokio::spawn(
                                async move { run_device_tracker(lib, cfg).await },
                            ));
                    }
                }
            } else if let Some(handle) = tracker_task.take() {
                tracing::info!("ADB disabled, stopping tracker task");
                handle.abort();
                registry::clear_connected_devices();
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
    library: LibraryService,
    settings: SettingsManager,
) -> Result<(), DeviceError> {
    let mut stream = adb::subscribe_track_devices().await?;
    tracing::info!("Successfully connected to ADB server and subscribed to device tracking");

    loop {
        let current = adb::read_device_list_update(&mut stream).await?;

        let adb_settings = {
            let s = settings.read().await;
            s.adb.clone()
        };

        let (connected_diff, disconnected_diff) = registry::update_connected_state(&current);

        for id in connected_diff {
            tracing::info!("ADB device connected: {id}");
            if let Some(cfg) = adb_settings.find_enabled_device(&id) {
                spawn_sync(id, cfg.remote_music_dir.clone(), library.clone());
            }
        }

        for id in disconnected_diff {
            tracing::info!("ADB device disconnected: {id}");
        }
    }
}

pub async fn sync_connected(library: LibraryService, settings: SettingsManager) {
    let s = settings.read().await;
    if !s.adb.enabled || !s.adb.auto_sync {
        return;
    }
    let adb_settings = s.adb.clone();
    drop(s);

    let connected = registry::get_connected_devices();
    for id in connected.iter() {
        if let Some(cfg) = adb_settings.find_enabled_device(id) {
            spawn_sync(
                id.to_string(),
                cfg.remote_music_dir.clone(),
                library.clone(),
            );
        }
    }
}

fn spawn_sync(device_id: String, remote_music_dir: String, library: LibraryService) {
    tokio::spawn(async move {
        if let Err(e) = engine::sync_device(&device_id, &remote_music_dir, library, false).await {
            tracing::error!("ADB device {device_id} sync failed: {e}");
        }
    });
}

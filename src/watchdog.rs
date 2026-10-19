use std::{path::Path, sync::Arc, time::Duration};

use anyhow::Result;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::RwLock;

use crate::{
    adb,
    api::download_manager::{DownloadManager, MessageLevel, ServerEvent},
    settings::SettingsManager,
    storage::MusicStorage,
    types::AudioFormat,
};

/// What: Sets up a filesystem watchdog using inotify/system events to monitor the music downloads folder.
/// Flow: Spawned at application startup to keep library state in sync with external file modifications.
/// Why:  Debounces incoming file events (waiting 2 seconds after the last write) so that large file copies
///       or downloads trigger a single clean re-scan, and dynamically rebinds if output_path changes in settings.
pub async fn init(
    storage: Arc<RwLock<MusicStorage>>,
    settings: SettingsManager,
    dm: Arc<DownloadManager>,
) -> Result<()> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        notify::Config::default(),
    )?;

    let path_to_watch = settings.read().await.downloads.output_path.clone();

    watcher.watch(Path::new(&path_to_watch), RecursiveMode::Recursive)?;
    tracing::info!("Watchdog monitoring directory: {}", path_to_watch);

    tokio::spawn(async move {
        let mut watcher = watcher;
        let mut path_to_watch = path_to_watch;
        let mut last_event_time = None;
        let debounce_duration = Duration::from_secs(2);
        let mut check_interval = tokio::time::interval(Duration::from_millis(500));
        let mut settings_rx = settings.subscribe_changes();

        loop {
            tokio::select! {
                Some(event) = rx.recv() => {
                    if is_relevant(&event) {
                        last_event_time = Some(tokio::time::Instant::now());
                    }
                }
                Ok(()) = settings_rx.changed() => {
                    let current_base_path = settings.read().await.downloads.output_path.clone();

                    if current_base_path != path_to_watch {
                        tracing::info!("Watchdog: base path changed from {} to {}", path_to_watch, current_base_path);
                        if let Err(e) = watcher.unwatch(Path::new(&path_to_watch)) {
                            tracing::warn!("Watchdog: failed to unwatch {}: {e}", path_to_watch);
                        }
                        let _ = std::fs::create_dir_all(&current_base_path);
                        match watcher.watch(Path::new(&current_base_path), RecursiveMode::Recursive) {
                            Ok(()) => {
                                path_to_watch = current_base_path;
                                tracing::info!("Watchdog: now monitoring directory: {}", path_to_watch);
                                last_event_time = Some(tokio::time::Instant::now());
                            }
                            Err(e) => {
                                tracing::error!("Watchdog: failed to watch new directory {}: {e}", current_base_path);
                            }
                        }
                    }
                }
                _ = check_interval.tick() => {
                    if last_event_time.is_some_and(|t| t.elapsed() >= debounce_duration) {
                        last_event_time = None;

                        if !dm.is_idle() {
                            tracing::debug!("Watchdog: filesystem changes detected, but downloads are active. Deferring sync.");
                            continue;
                        }

                        tracing::info!("Watchdog: changes detected, reindexing library");

                        dm.broadcast_event(ServerEvent::Message {
                            message: "Library changes detected, reindexing...".to_string(),
                            level: MessageLevel::Info,
                        });

                        let tracks = MusicStorage::scan_library(&path_to_watch).await;
                        storage.write().await.update_tracks(tracks);
                        adb::sync_connected(storage.clone(), settings.clone()).await;

                        dm.broadcast_event(ServerEvent::Message {
                            message: "Library reindexing and sync completed".to_string(),
                            level: MessageLevel::Info,
                        });
                        dm.broadcast_event(ServerEvent::SyncFinished { url: None });
                    }
                }
            }
        }
    });

    Ok(())
}

fn is_relevant(event: &notify::Event) -> bool {
    let is_mutation = event.kind.is_create() || event.kind.is_modify() || event.kind.is_remove();
    if !is_mutation {
        return false;
    }

    event
        .paths
        .iter()
        .any(|path| AudioFormat::from_path(path) != AudioFormat::Unknown)
}

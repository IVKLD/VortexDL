use std::{path::Path, sync::Arc, time::Duration};

use anyhow::Result;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::RwLock;

use crate::{
    adb_device,
    api::{
        download_manager::{DownloadManager, ServerEvent},
        types::AudioFormat,
    },
    settings::SettingsManager,
    storage::MusicStorage,
};

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

    let path_to_watch = settings.output_path().await;

    watcher.watch(Path::new(&path_to_watch), RecursiveMode::Recursive)?;
    tracing::info!("Watchdog monitoring directory: {}", path_to_watch);

    tokio::spawn(async move {
        let mut watcher = watcher;
        let mut path_to_watch = path_to_watch;
        let mut last_event_time = None;
        let debounce_duration = Duration::from_secs(2);
        let mut check_interval = tokio::time::interval(Duration::from_millis(500));

        loop {
            tokio::select! {
                Some(event) = rx.recv() => {
                    if is_relevant(&event) {
                        last_event_time = Some(tokio::time::Instant::now());
                    }
                }
                _ = check_interval.tick() => {
                    let current_base_path = settings.output_path().await;

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

                    if last_event_time.is_some_and(|t| t.elapsed() >= debounce_duration) {
                        last_event_time = None;
                        tracing::info!("Watchdog: changes detected, reindexing library");

                        dm.broadcast_event(ServerEvent::Message {
                            message: "Library changes detected, reindexing...".to_string(),
                            level: "info".to_string(),
                        });

                        let tracks = MusicStorage::scan_library(&path_to_watch).await;
                        storage.write().await.update_tracks(tracks);
                        adb_device::sync_connected(storage.clone(), settings.clone()).await;

                        dm.broadcast_event(ServerEvent::Message {
                            message: "Library reindexing and sync completed".to_string(),
                            level: "info".to_string(),
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
        .any(|path| matches!(AudioFormat::from_path(path), AudioFormat::Mp3))
}

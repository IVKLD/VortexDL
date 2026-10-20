use std::{path::Path, sync::Arc, time::Duration};

use anyhow::Result;
use notify::{
    Config as WatcherConfig, Event, RecommendedWatcher, RecursiveMode, Result as NotifyResult,
    Watcher,
};
use tokio::sync::{RwLock, mpsc::unbounded_channel};

use crate::{
    devices,
    domain::AudioFormat,
    downloader::{DownloadManager, MessageLevel, ServerEvent},
    library::{Library, scan_directory},
    settings::SettingsManager,
};

/// Sets up a filesystem watcher using inotify/system events to monitor the music downloads folder.
/// Debounces incoming file events so that completed downloads/copies trigger a single clean re-scan,
/// and dynamically rebinds if output_path changes in settings.
pub async fn start_library_watcher(
    storage: Arc<RwLock<Library>>,
    settings: SettingsManager,
    dm: Arc<DownloadManager>,
) -> Result<()> {
    let (tx, mut rx) = unbounded_channel();

    let mut watcher = RecommendedWatcher::new(
        move |res: NotifyResult<Event>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        WatcherConfig::default(),
    )?;

    let path_to_watch = settings.read().await.downloads.output_path.clone();

    watcher.watch(Path::new(&path_to_watch), RecursiveMode::Recursive)?;
    tracing::info!("Library watcher monitoring directory: {}", path_to_watch);

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
                        tracing::info!("Library watcher: base path changed from {} to {}", path_to_watch, current_base_path);
                        if let Err(e) = watcher.unwatch(Path::new(&path_to_watch)) {
                            tracing::warn!("Library watcher: failed to unwatch {}: {e}", path_to_watch);
                        }
                        let _ = std::fs::create_dir_all(&current_base_path);
                        match watcher.watch(Path::new(&current_base_path), RecursiveMode::Recursive) {
                            Ok(()) => {
                                path_to_watch = current_base_path;
                                tracing::info!("Library watcher: now monitoring directory: {}", path_to_watch);
                                last_event_time = Some(tokio::time::Instant::now());
                            }
                            Err(e) => {
                                tracing::error!("Library watcher: failed to watch new directory {}: {e}", current_base_path);
                            }
                        }
                    }
                }
                _ = check_interval.tick() => {
                    if last_event_time.is_some_and(|t| t.elapsed() >= debounce_duration) {
                        last_event_time = None;

                        if !dm.is_idle() {
                            tracing::debug!("Library watcher: filesystem changes detected, but downloads are active. Deferring sync.");
                            continue;
                        }

                        tracing::info!("Library watcher: changes detected, reindexing library");

                        dm.broadcast_event(ServerEvent::Message {
                            message: "Library changes detected, reindexing...".to_string(),
                            level: MessageLevel::Info,
                        });

                        let tracks = scan_directory(&path_to_watch).await;
                        storage.write().await.update_tracks(tracks);
                        devices::sync_connected(storage.clone(), settings.clone()).await;

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

fn is_relevant(event: &Event) -> bool {
    let is_mutation = event.kind.is_create() || event.kind.is_modify() || event.kind.is_remove();
    if !is_mutation {
        return false;
    }

    event.paths.iter().any(|path| {
        if path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|name| name.starts_with('.'))
        {
            return false;
        }

        AudioFormat::from_path(path) != AudioFormat::Unknown
    })
}

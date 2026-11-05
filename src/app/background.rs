use std::{fs, path::Path, sync::Arc, time::Duration};

use anyhow::Result;
use notify::{
    Config as WatcherConfig, Event, RecommendedWatcher, RecursiveMode, Result as NotifyResult,
    Watcher,
};
use tokio::sync::mpsc;

use crate::{
    devices::DeviceService,
    domain::AudioFormat,
    downloader::{DownloadQueue, MessageLevel, ServerEvent},
    infra::EventBus,
    library::LibraryService,
    settings::SettingsManager,
};

/// Sets up a filesystem watcher using inotify/system events to monitor the music downloads folder.
/// Debounces incoming file events so that completed downloads/copies trigger a single clean re-scan,
/// and dynamically rebinds if output_path changes in settings.
pub async fn start_library_watcher(
    library: LibraryService,
    devices: DeviceService,
    settings: SettingsManager,
    queue: Arc<DownloadQueue>,
    events: EventBus,
) -> Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel();

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
        let debounce_duration = Duration::from_secs(2);
        let mut debounce_timer = std::pin::pin!(tokio::time::sleep(debounce_duration));
        let mut has_pending_events = false;
        let mut settings_rx = settings.subscribe_changes();

        loop {
            tokio::select! {
                Some(event) = rx.recv() => {
                    if is_relevant(&event) {
                        debounce_timer.as_mut().reset(tokio::time::Instant::now() + debounce_duration);
                        has_pending_events = true;
                    }
                }
                Ok(()) = settings_rx.changed() => {
                    let current_base_path = settings.read().await.downloads.output_path.clone();

                    if current_base_path != path_to_watch {
                        tracing::info!("Library watcher: base path changed from {} to {}", path_to_watch, current_base_path);
                        if let Err(e) = watcher.unwatch(Path::new(&path_to_watch)) {
                            tracing::warn!("Library watcher: failed to unwatch {}: {e}", path_to_watch);
                        }
                        let _ = fs::create_dir_all(&current_base_path);
                        match watcher.watch(Path::new(&current_base_path), RecursiveMode::Recursive) {
                            Ok(()) => {
                                path_to_watch = current_base_path;
                                tracing::info!("Library watcher: now monitoring directory: {}", path_to_watch);
                                debounce_timer.as_mut().reset(tokio::time::Instant::now() + debounce_duration);
                                has_pending_events = true;
                            }
                            Err(e) => {
                                tracing::error!("Library watcher: failed to watch new directory {}: {e}", current_base_path);
                            }
                        }
                    }
                }
                () = &mut debounce_timer, if has_pending_events => {
                    if !queue.is_idle() {
                        tracing::debug!("Library watcher: filesystem changes detected, but downloads are active. Deferring sync.");
                        debounce_timer.as_mut().reset(tokio::time::Instant::now() + debounce_duration);
                        continue;
                    }

                    has_pending_events = false;

                    tracing::info!("Library watcher: changes detected, reindexing library");

                    events.publish(ServerEvent::Message {
                        message: "Library changes detected, reindexing...".to_string(),
                        level: MessageLevel::Info,
                    });

                    library.reindex().await;
                    devices.sync_connected().await;

                    events.publish(ServerEvent::Message {
                        message: "Library reindexing and sync completed".to_string(),
                        level: MessageLevel::Info,
                    });
                    events.publish(ServerEvent::SyncFinished { url: None });
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

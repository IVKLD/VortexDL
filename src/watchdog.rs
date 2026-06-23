use std::{path::Path, sync::Arc, time::Duration};
use anyhow::Result;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::RwLock;

use crate::{
    adb_device,
    api::download_manager::{DownloadManager, ServerEvent},
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

    let path_to_watch = {
        let s = storage.read().await;
        s.base_path.clone()
    };

    watcher.watch(Path::new(&path_to_watch), RecursiveMode::Recursive)?;
    tracing::info!("Watchdog monitoring directory: {}", path_to_watch);

    tokio::spawn(async move {
        let _watcher_keep_alive = watcher;
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
                    if let Some(t) = last_event_time {
                        if t.elapsed() >= debounce_duration {
                            last_event_time = None;
                            tracing::info!("Watchdog: changes detected, reindexing library");
                            MusicStorage::index_library(storage.clone()).await;
                            adb_device::sync_connected(storage.clone(), settings.clone()).await;
                            dm.broadcast_event(ServerEvent::SyncFinished);
                        }
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

    event.paths.iter().any(|path| {
        path.extension()
            .map_or(false, |ext| ext.to_string_lossy().to_lowercase() == "mp3")
    })
}

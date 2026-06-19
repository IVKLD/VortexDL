use std::{
    collections::{HashMap, HashSet},
    fs, io,
    path::{Path, PathBuf},
    sync::Arc,
    time::UNIX_EPOCH,
};

use anyhow::Result;
use tokio::{sync::RwLock, task::spawn_blocking};

use crate::{
    database::{get_previous_ids, save_sync_ids},
    types::SyncMode,
    utils::metadata::extract_track_metadata,
};

#[derive(Default, Clone)]
pub struct TrackData {
    pub path: PathBuf,
    pub artist: String,
    pub title: String,
    pub artwork_url: Option<String>,
    pub source_url: Option<String>,
    pub position: Option<u32>,
    pub created_at: u64,
    pub size: u64,
}

impl TrackData {
    pub fn is_archived(&self) -> bool {
        self.path.iter().any(|c| c == "Archive")
    }
}

#[derive(Clone)]
pub struct MusicStorage {
    pub base_path: String,
    pub tracks: HashMap<i64, TrackData>,
}

impl MusicStorage {
    pub fn new(base_path: String) -> Self {
        Self {
            base_path,
            tracks: HashMap::new(),
        }
    }
    pub fn update_track(&mut self, id: i64, data: TrackData) {
        self.tracks.insert(id, data);
    }
    pub fn remove_track(&mut self, id: i64) -> Result<Option<TrackData>, io::Error> {
        if let Some(data) = self.tracks.remove(&id) {
            if data.path.exists() {
                fs::remove_file(&data.path)?;
            }
            Ok(Some(data))
        } else {
            Ok(None)
        }
    }
    pub async fn run_background_indexing(storage: Arc<RwLock<Self>>) {
        let root = {
            let s = storage.read().await;
            PathBuf::from(&s.base_path)
        };

        let result = spawn_blocking(move || {
            let mut new_tracks = HashMap::new();
            let mut stack = vec![root];

            while let Some(dir) = stack.pop() {
                let Ok(entries) = fs::read_dir(dir) else {
                    continue;
                };

                for entry in entries.flatten() {
                    let path = entry.path();

                    if path.is_dir() {
                        stack.push(path);
                        continue;
                    }

                    let Some(meta) = extract_track_metadata(&path) else {
                        continue;
                    };

                    let (created_at, size) = entry.metadata().ok().map_or((0, 0), |m| {
                        let created = m
                            .created()
                            .ok()
                            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                            .unwrap_or_default()
                            .as_secs();
                        (created, m.len())
                    });

                    new_tracks.insert(
                        meta.id,
                        TrackData {
                            path,
                            artist: meta.artist,
                            title: meta.title,
                            artwork_url: meta.artwork_url,
                            source_url: meta.source_url,
                            position: meta.position,
                            created_at,
                            size,
                        },
                    );
                }
            }
            new_tracks
        })
        .await;

        if let Ok(new_tracks) = result {
            let mut s = storage.write().await;
            let count = new_tracks.len();

            s.tracks = new_tracks;

            tracing::info!("Indexing complete. Found {} tracks.", count);
        }
    }

    pub async fn sync_storage(
        &mut self,
        url: &str,
        remote_ids: &HashSet<i64>,
        mode: &SyncMode,
    ) -> Result<()> {
        let previous_ids = get_previous_ids(url)?;

        let remove_ids: Vec<i64> = previous_ids.difference(remote_ids).copied().collect();

        if remove_ids.is_empty() || matches!(mode, SyncMode::Silent) {
            save_sync_ids(url, remote_ids)?;
            return Ok(());
        }

        let archive_path = Path::new(&self.base_path).join("Archive");
        if matches!(mode, SyncMode::Archive) {
            tokio::fs::create_dir_all(&archive_path).await?;
        }

        for id in remove_ids {
            if let Some(data) = self.tracks.remove(&id).filter(|d| d.path.exists()) {
                match mode {
                    SyncMode::Full => {
                        tokio::fs::remove_file(&data.path).await?;
                    }
                    SyncMode::Archive => {
                        if let Some(name) = data.path.file_name() {
                            tokio::fs::rename(&data.path, archive_path.join(name)).await?;
                        }
                    }
                    _ => {}
                }
            }
        }

        save_sync_ids(url, remote_ids)?;
        Ok(())
    }
}

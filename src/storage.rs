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
    database::{
        cache::{CachedTrack, get_cached_tracks, save_cached_tracks},
        get_previous_ids, save_sync_ids,
    },
    types::SyncMode,
    utils::metadata::extract_track_metadata,
};

#[derive(Default, Clone)]
pub struct LocalTrack {
    pub path: PathBuf,
    pub artist: String,
    pub title: String,
    pub artwork_url: Option<String>,
    pub source_url: Option<String>,
    pub position: Option<u32>,
    pub created_at: u64,
    pub size: u64,
}

impl LocalTrack {
    pub fn is_archived(&self) -> bool {
        self.path.iter().any(|c| c == "Archive")
    }

    pub fn from_cached(path: PathBuf, cached: &CachedTrack) -> Self {
        Self {
            path,
            artist: cached.artist.clone(),
            title: cached.title.clone(),
            artwork_url: cached.artwork_url.clone(),
            source_url: cached.source_url.clone(),
            position: cached.position,
            created_at: cached.created_at,
            size: cached.size,
        }
    }
}

fn process_file(
    path: PathBuf,
    metadata: &fs::Metadata,
    cached_map: &HashMap<String, CachedTrack>,
) -> Option<(i64, LocalTrack, String, CachedTrack)> {
    let size = metadata.len();
    let get_secs = |t: std::time::SystemTime| {
        t.duration_since(UNIX_EPOCH)
            .ok()
            .map(|d| d.as_secs())
            .unwrap_or_default()
    };

    let mtime = metadata.modified().ok().map(get_secs).unwrap_or_default();
    let created_at = metadata.created().ok().map(get_secs).unwrap_or_default();
    let path_str = path.to_string_lossy().to_string();

    if let Some(cached) = cached_map
        .get(&path_str)
        .filter(|c| c.size == size && c.mtime == mtime)
    {
        let data = LocalTrack::from_cached(path, cached);
        return Some((cached.id, data, path_str, cached.clone()));
    }

    let meta = extract_track_metadata(&path)?;
    let cached = CachedTrack {
        id: meta.id,
        artist: meta.artist,
        title: meta.title,
        artwork_url: meta.artwork_url,
        source_url: meta.source_url,
        position: meta.position,
        created_at,
        size,
        mtime,
    };
    let data = LocalTrack::from_cached(path, &cached);
    Some((cached.id, data, path_str, cached))
}

#[derive(Default)]
pub struct MusicStorage {
    pub base_path: String,
    pub tracks: HashMap<i64, LocalTrack>,
}

impl MusicStorage {
    pub fn new(base_path: String) -> Self {
        Self {
            base_path,
            tracks: HashMap::new(),
        }
    }
    pub fn update_track(&mut self, id: i64, data: LocalTrack) {
        self.tracks.insert(id, data);
    }
    pub async fn remove_track(&mut self, id: i64) -> Result<Option<LocalTrack>, io::Error> {
        let Some(data) = self.tracks.remove(&id) else {
            return Ok(None);
        };
        if data.path.exists() {
            tokio::fs::remove_file(&data.path).await?;
        }
        Ok(Some(data))
    }
    pub async fn index_library(storage: Arc<RwLock<Self>>) {
        let root = {
            let s = storage.read().await;
            PathBuf::from(&s.base_path)
        };

        let result = spawn_blocking(move || {
            let mut tracks = HashMap::new();
            let mut stack = vec![root];
            let cached = get_cached_tracks().unwrap_or_default();
            let mut new_cached = HashMap::new();

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

                    let Ok(metadata) = entry.metadata() else {
                        continue;
                    };

                    if let Some((id, data, path_str, cache_item)) =
                        process_file(path, &metadata, &cached)
                    {
                        new_cached.insert(path_str, cache_item);
                        tracks.insert(id, data);
                    }
                }
            }

            if let Err(e) = save_cached_tracks(&new_cached) {
                tracing::error!("Failed to save track cache: {e}");
            }

            tracks
        })
        .await;

        if let Ok(mut tracks) = result {
            let mut s = storage.write().await;
            for (id, data) in &s.tracks {
                if !tracks.contains_key(id) && data.path.exists() {
                    tracks.insert(*id, data.clone());
                }
            }
            let count = tracks.len();
            s.tracks = tracks;
            tracing::info!("Indexing complete. Found {} tracks.", count);
        }
    }

    pub async fn sync_storage(
        &mut self,
        url: &str,
        current_soundcloud_ids: &HashSet<i64>,
        mode: &SyncMode,
    ) -> Result<()> {
        let prev_ids = get_previous_ids(url)?;
        let to_remove: Vec<i64> = prev_ids
            .difference(current_soundcloud_ids)
            .copied()
            .collect();

        if to_remove.is_empty() || matches!(mode, SyncMode::Silent) {
            save_sync_ids(url, current_soundcloud_ids)?;
            return Ok(());
        }

        let archive_path = Path::new(&self.base_path).join("Archive");
        if matches!(mode, SyncMode::Archive) {
            tokio::fs::create_dir_all(&archive_path).await?;
        }

        for id in to_remove {
            match self.tracks.remove(&id) {
                Some(data) if data.path.exists() => match mode {
                    SyncMode::Full => {
                        tokio::fs::remove_file(&data.path).await?;
                    }
                    SyncMode::Archive => {
                        if let Some(name) = data.path.file_name() {
                            tokio::fs::rename(&data.path, archive_path.join(name)).await?;
                        }
                    }
                    _ => {}
                },
                Some(data) => {
                    tracing::warn!(
                        id,
                        path = %data.path.display(),
                        "Track removed from remote but file already missing on disk"
                    );
                }
                None => {}
            }
        }

        save_sync_ids(url, current_soundcloud_ids)?;
        Ok(())
    }
}

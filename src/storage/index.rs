use std::{collections::HashMap, fs, path::PathBuf, sync::Arc, time::UNIX_EPOCH};

use tokio::{sync::RwLock, task::spawn_blocking};

use super::{LocalMusicTrack, MusicStorage};
use crate::{
    api::types::AudioFormat,
    database::cache::{CachedMusicTrack, get_cached_music_tracks, update_cached_tracks_batch},
    utils::metadata::extract_track_metadata,
};

impl MusicStorage {
    pub async fn index_library(storage: Arc<RwLock<Self>>) {
        let root = {
            let s = storage.read().await;
            PathBuf::from(&s.base_path)
        };

        let result = spawn_blocking(move || {
            let mut tracks = HashMap::new();
            let mut stack = vec![root];
            let cached = get_cached_music_tracks().unwrap_or_default();
            let mut visited = std::collections::HashSet::new();
            let mut to_update = HashMap::new();

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

                    if !matches!(AudioFormat::from_path(&path), AudioFormat::Mp3) {
                        continue;
                    }

                    let Ok(metadata) = entry.metadata() else {
                        continue;
                    };

                    let size = metadata.len();
                    let (mtime, created_at) = get_file_timestamps(&metadata);
                    let path_str = path.to_string_lossy().into_owned();

                    if let Some(cached_track) = cached
                        .get(&path_str)
                        .filter(|c| c.size == size && c.mtime == mtime)
                    {
                        visited.insert(path_str);
                        let data = LocalMusicTrack::from_cached(path, cached_track);
                        tracks.insert(cached_track.id, data);
                    } else if let Some(new_cached) =
                        parse_file_metadata(&path, size, mtime, created_at)
                    {
                        visited.insert(path_str.clone());
                        let data = LocalMusicTrack::from_cached(path, &new_cached);
                        tracks.insert(new_cached.id, data);
                        to_update.insert(path_str, new_cached);
                    }
                }
            }

            let mut to_remove = std::collections::HashSet::new();
            for path in cached.keys() {
                if !visited.contains(path) {
                    to_remove.insert(path.clone());
                }
            }

            if let Err(e) = update_cached_tracks_batch(&to_update, &to_remove) {
                tracing::error!("Failed to update track cache batch: {e}");
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
            tracing::debug!("Indexing complete. Found {} tracks.", count);
        }
    }
}

fn system_time_to_secs(t: std::time::SystemTime) -> u64 {
    t.duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn get_file_timestamps(metadata: &fs::Metadata) -> (u64, u64) {
    let mtime = metadata
        .modified()
        .ok()
        .map(system_time_to_secs)
        .unwrap_or(0);
    let mut created_at = metadata
        .created()
        .ok()
        .map(system_time_to_secs)
        .unwrap_or(0);
    if created_at == 0 {
        created_at = mtime;
    }
    (mtime, created_at)
}

fn parse_file_metadata(
    path: &PathBuf,
    size: u64,
    mtime: u64,
    created_at: u64,
) -> Option<CachedMusicTrack> {
    let meta = extract_track_metadata(path)?;
    Some(CachedMusicTrack {
        id: meta.id,
        artist: meta.artist,
        title: meta.title,
        artwork_url: meta.artwork_url,
        source_url: meta.source_url,
        position: meta.position,
        created_at,
        size,
        mtime,
    })
}

use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
};

use tokio::task::spawn_blocking;

use super::{MusicStorage, metadata::extract_track_metadata, model::LocalMusicTrack};
use crate::{
    api::types::AudioFormat,
    database::cache::{CachedMusicTrack, get_cached_music_tracks, update_cached_tracks_batch},
    utils::time::system_time_to_secs,
};

fn get_file_timestamps(metadata: &fs::Metadata) -> (u64, u64) {
    let mtime = metadata.modified().ok().map(system_time_to_secs).unwrap_or(0);
    let mut created_at = metadata.created().ok().map(system_time_to_secs).unwrap_or(0);
    if created_at == 0 {
        created_at = mtime;
    }
    (mtime, created_at)
}

impl MusicStorage {
    pub async fn scan_library(base_path: &str) -> HashMap<i64, LocalMusicTrack> {
        let root = PathBuf::from(base_path);

        let result = spawn_blocking(move || {
            let mut tracks: HashMap<i64, LocalMusicTrack> = HashMap::new();
            let mut insert_track = |id: i64, data: LocalMusicTrack| {
                if tracks
                    .get(&id)
                    .is_some_and(|existing| !existing.is_archived() && data.is_archived())
                {
                    return;
                }
                tracks.insert(id, data);
            };
            let mut stack = vec![root];
            let cached = get_cached_music_tracks().unwrap_or_default();
            let mut visited = HashSet::new();
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

                    if AudioFormat::from_path(&path) == AudioFormat::Unknown {
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
                        insert_track(cached_track.metadata.id, data);
                    } else if let Some(new_cached) =
                        parse_file_metadata(&path, size, mtime, created_at)
                    {
                        visited.insert(path_str.clone());
                        let data = LocalMusicTrack::from_cached(path, &new_cached);
                        insert_track(new_cached.metadata.id, data);
                        to_update.insert(path_str, new_cached);
                    }
                }
            }

            let to_remove: HashSet<_> = cached
                .keys()
                .filter(|path| !visited.contains(*path))
                .cloned()
                .collect();

            if let Err(e) = update_cached_tracks_batch(&to_update, &to_remove) {
                tracing::error!("Failed to update track cache batch: {e}");
            }

            tracks
        })
        .await;

        result.unwrap_or_default()
    }
}



fn parse_file_metadata(
    path: &PathBuf,
    size: u64,
    mtime: u64,
    created_at: u64,
) -> Option<CachedMusicTrack> {
    let metadata = extract_track_metadata(path)?;
    Some(CachedMusicTrack {
        metadata,
        created_at,
        size,
        mtime,
    })
}

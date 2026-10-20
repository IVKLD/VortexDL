use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use tokio::task::spawn_blocking;

use super::{ARCHIVE_DIR, metadata::extract_track_metadata, types::LibraryTrack};
use crate::{
    domain::AudioFormat,
    infra::{
        db::cache::{CachedTrack, get_cached_tracks, update_cached_tracks_batch},
        system_time_to_secs,
    },
};

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

/// What: Scans the local music directory on disk and populates the library track cache.
/// Flow: Runs at application startup and when the user requests a manual library re-index via API.
/// Why:  Reading audio tags on thousands of files is slow. This checks file modification times against
///       the database cache table to skip files that haven't changed since the last scan.
pub async fn scan_directory(base_path: impl AsRef<Path>) -> HashMap<i64, LibraryTrack> {
    let root = base_path.as_ref().to_path_buf();

    let result = spawn_blocking(move || {
        let mut tracks: HashMap<i64, LibraryTrack> = HashMap::new();
        let mut insert_track = |id: i64, data: LibraryTrack| {
            if tracks
                .get(&id)
                .is_some_and(|existing| !existing.is_archived && data.is_archived)
            {
                return;
            }
            tracks.insert(id, data);
        };
        let mut stack = vec![root.clone()];
        let cached = get_cached_tracks().unwrap_or_default();
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

                let is_archived = path
                    .strip_prefix(&root)
                    .map(|rel| rel.iter().any(|c| c == ARCHIVE_DIR))
                    .unwrap_or(false);

                if let Some(cached_track) = cached
                    .get(&path_str)
                    .filter(|c| c.size == size && c.mtime == mtime)
                {
                    visited.insert(path_str);
                    let data = LibraryTrack::from_cached(path, cached_track, is_archived);
                    insert_track(data.metadata.id, data);
                } else if let Some(new_cached) = parse_file_metadata(&path, size, mtime, created_at)
                {
                    visited.insert(path_str.clone());
                    let data = LibraryTrack::from_cached(path, &new_cached, is_archived);
                    insert_track(data.metadata.id, data);
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

fn parse_file_metadata(
    path: &PathBuf,
    size: u64,
    mtime: u64,
    created_at: u64,
) -> Option<CachedTrack> {
    let metadata = extract_track_metadata(path)?;
    Some(CachedTrack {
        metadata,
        created_at,
        size,
        mtime,
    })
}

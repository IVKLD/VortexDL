use std::path::PathBuf;

use crate::{constants::ARCHIVE_DIR, database::cache::CachedMusicTrack, types::TrackMetadata};

#[derive(Default, Clone, Debug)]
pub struct LocalMusicTrack {
    pub path: PathBuf,
    pub metadata: TrackMetadata,
    pub created_at: u64,
    pub size: u64,
}

impl LocalMusicTrack {
    pub fn is_archived(&self) -> bool {
        self.path.iter().any(|c| c == ARCHIVE_DIR)
    }

    pub fn from_cached(path: PathBuf, cached: &CachedMusicTrack) -> Self {
        Self {
            path,
            metadata: cached.metadata.clone(),
            created_at: cached.created_at,
            size: cached.size,
        }
    }
}

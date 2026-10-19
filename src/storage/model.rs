use std::path::PathBuf;

use crate::{database::cache::CachedMusicTrack, types::TrackMetadata};

#[derive(Default, Clone, Debug)]
pub struct LocalMusicTrack {
    pub path: PathBuf,
    pub metadata: TrackMetadata,
    pub created_at: u64,
    pub size: u64,
    pub is_archived: bool,
}

impl LocalMusicTrack {
    pub fn from_cached(path: PathBuf, cached: &CachedMusicTrack, is_archived: bool) -> Self {
        Self {
            path,
            metadata: cached.metadata.clone(),
            created_at: cached.created_at,
            size: cached.size,
            is_archived,
        }
    }
}

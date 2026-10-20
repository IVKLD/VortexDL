use std::path::PathBuf;

use crate::{domain::TrackMetadata, infra::db::CachedTrack};

#[derive(Default, Clone, Debug)]
pub struct LibraryTrack {
    pub path: PathBuf,
    pub metadata: TrackMetadata,
    pub created_at: u64,
    pub size: u64,
    pub is_archived: bool,
}

impl LibraryTrack {
    pub fn from_cached(path: PathBuf, cached: &CachedTrack, is_archived: bool) -> Self {
        Self {
            path,
            metadata: cached.metadata.clone(),
            created_at: cached.created_at,
            size: cached.size,
            is_archived,
        }
    }
}

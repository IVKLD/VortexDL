use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    domain::TrackMetadata,
    infra::db::{CachedTrack, DeletedTrackRecord},
};

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

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LibraryStats {
    pub total_tracks: usize,
    pub total_size: u64,
    pub total_downloaded: u64,
    pub total_downloaded_bytes: u64,
    pub total_deleted: u64,
    pub total_deleted_bytes: u64,
    pub recent_deleted: Vec<DeletedTrackRecord>,
}

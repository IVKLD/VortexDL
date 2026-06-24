use std::path::PathBuf;

use crate::{constants::ARCHIVE_DIR, database::cache::CachedMusicTrack};

#[derive(Default, Clone, Debug)]
pub struct LocalMusicTrack {
    pub path: PathBuf,
    pub artist: String,
    pub title: String,
    pub artwork_url: Option<String>,
    pub source_url: Option<String>,
    pub position: Option<u32>,
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

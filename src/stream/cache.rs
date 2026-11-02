use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::{infra::MutexExt, providers::HlsPlaylist};

#[derive(Clone, Debug)]
pub struct CachedStream {
    pub url: String,
    pub proxy: Option<String>,
    pub hls_playlist: Option<HlsPlaylist>,
    pub instant: Instant,
}

#[derive(Default, Clone)]
pub struct StreamCache {
    streams: Arc<Mutex<HashMap<i64, CachedStream>>>,
}

impl StreamCache {
    pub fn get(&self, id: i64) -> Option<CachedStream> {
        let streams = self.streams.lock_unpoisoned();
        let entry = streams.get(&id)?;
        (entry.instant.elapsed() < Duration::from_secs(3 * 3600)).then(|| entry.clone())
    }

    pub fn insert(&self, id: i64, url: String, proxy: Option<String>) {
        self.streams.lock_unpoisoned().insert(
            id,
            CachedStream {
                url,
                proxy,
                hls_playlist: None,
                instant: Instant::now(),
            },
        );
    }

    pub fn set_hls_playlist(&self, id: i64, playlist: HlsPlaylist) {
        if let Some(entry) = self.streams.lock_unpoisoned().get_mut(&id) {
            entry.hls_playlist = Some(playlist);
        }
    }

    pub fn remove(&self, id: i64) {
        self.streams.lock_unpoisoned().remove(&id);
    }
}

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use soundcloud_rs::Track;

use crate::infra::MutexExt;

#[derive(Default)]
struct SearchCacheInner {
    youtube_ids: HashMap<i64, String>,
    soundcloud_tracks: HashMap<i64, Track>,
    continuations: HashMap<String, String>,
}

#[derive(Default, Clone)]
pub struct SearchCache {
    inner: Arc<Mutex<SearchCacheInner>>,
}

impl SearchCache {
    pub fn get_youtube_id(&self, id: i64) -> Option<String> {
        self.inner.lock_unpoisoned().youtube_ids.get(&id).cloned()
    }

    pub fn insert_youtube_id(&self, id: i64, vid: String) {
        self.inner.lock_unpoisoned().youtube_ids.insert(id, vid);
    }

    pub fn get_soundcloud_track(&self, id: i64) -> Option<Track> {
        self.inner
            .lock_unpoisoned()
            .soundcloud_tracks
            .get(&id)
            .cloned()
    }

    pub fn insert_soundcloud_track(&self, id: i64, track: Track) {
        self.inner
            .lock_unpoisoned()
            .soundcloud_tracks
            .insert(id, track);
    }

    pub fn continuation(&self, query: &str) -> Option<String> {
        self.inner
            .lock_unpoisoned()
            .continuations
            .get(&query.to_lowercase())
            .cloned()
    }

    pub fn set_continuation(&self, query: &str, token: Option<String>) {
        let mut inner = self.inner.lock_unpoisoned();
        let key = query.to_lowercase();
        if let Some(token) = token {
            inner.continuations.insert(key, token);
        } else {
            inner.continuations.remove(&key);
        }
    }
}

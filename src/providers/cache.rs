use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use reqwest::Client;
use soundcloud_rs::Track;
use yt_audio_downloader::create_http_client_with_proxy;

use crate::infra::MutexExt;

#[derive(Clone, Debug)]
pub struct CachedStream {
    pub url: String,
    pub proxy: Option<String>,
    pub instant: Instant,
}

#[derive(Default)]
struct CacheInner {
    youtube_ids: HashMap<i64, String>,
    streams: HashMap<i64, CachedStream>,
    soundcloud_tracks: HashMap<i64, Track>,
    continuations: HashMap<String, String>,
    active_proxy: Option<String>,
    clients: HashMap<Option<String>, Client>,
}

#[derive(Default, Clone)]
pub struct ProviderCache {
    inner: Arc<Mutex<CacheInner>>,
}

impl ProviderCache {
    pub fn get_youtube_id(&self, id: i64) -> Option<String> {
        self.inner.lock_unpoisoned().youtube_ids.get(&id).cloned()
    }

    pub fn insert_youtube_id(&self, id: i64, vid: String) {
        self.inner.lock_unpoisoned().youtube_ids.insert(id, vid);
    }

    pub fn get_stream(&self, id: i64) -> Option<CachedStream> {
        let inner = self.inner.lock_unpoisoned();
        let entry = inner.streams.get(&id)?;
        (entry.instant.elapsed() < Duration::from_secs(3 * 3600)).then(|| entry.clone())
    }

    pub fn insert_stream(&self, id: i64, url: String, proxy: Option<String>) {
        self.inner.lock_unpoisoned().streams.insert(
            id,
            CachedStream {
                url,
                proxy,
                instant: Instant::now(),
            },
        );
    }

    pub fn remove_stream(&self, id: i64) {
        self.inner.lock_unpoisoned().streams.remove(&id);
    }

    pub fn active_proxy(&self) -> Option<String> {
        self.inner.lock_unpoisoned().active_proxy.clone()
    }

    pub fn set_active_proxy(&self, proxy: Option<String>) {
        self.inner.lock_unpoisoned().active_proxy = proxy;
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
        if let Some(tok) = token {
            inner.continuations.insert(key, tok);
        } else {
            inner.continuations.remove(&key);
        }
    }

    pub fn get_or_create_client(&self, proxy: Option<String>) -> Client {
        self.inner
            .lock_unpoisoned()
            .clients
            .entry(proxy.clone())
            .or_insert_with(|| create_http_client_with_proxy(proxy.as_deref()))
            .clone()
    }
}

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use reqwest::Client;
use soundcloud_rs::Track;
use tokio::sync::RwLock;
use yt_audio_downloader::create_http_client_with_proxy;

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
    search_continuations: HashMap<String, String>,
    active_proxy: Option<String>,
    http_clients: HashMap<Option<String>, Client>,
}

#[derive(Default, Clone)]
pub struct AppCache {
    inner: Arc<RwLock<CacheInner>>,
}

impl AppCache {
    pub async fn get_youtube_id(&self, id: i64) -> Option<String> {
        self.inner.read().await.youtube_ids.get(&id).cloned()
    }

    pub async fn insert_youtube_id(&self, id: i64, vid: String) {
        self.inner.write().await.youtube_ids.insert(id, vid);
    }

    pub async fn get_stream(&self, id: i64) -> Option<CachedStream> {
        let read = self.inner.read().await;
        if let Some(entry) = read.streams.get(&id)
            && entry.instant.elapsed() < Duration::from_secs(3 * 3600)
        {
            return Some(entry.clone());
        }
        None
    }

    pub async fn insert_stream(&self, id: i64, url: String, proxy: Option<String>) {
        self.inner.write().await.streams.insert(
            id,
            CachedStream {
                url,
                proxy,
                instant: Instant::now(),
            },
        );
    }

    pub async fn remove_stream(&self, id: i64) {
        self.inner.write().await.streams.remove(&id);
    }

    pub async fn get_active_proxy(&self) -> Option<String> {
        self.inner.read().await.active_proxy.clone()
    }

    pub async fn set_active_proxy(&self, proxy: Option<String>) {
        self.inner.write().await.active_proxy = proxy;
    }

    pub async fn get_soundcloud_track(&self, id: i64) -> Option<soundcloud_rs::Track> {
        self.inner.read().await.soundcloud_tracks.get(&id).cloned()
    }

    pub async fn insert_soundcloud_track(&self, id: i64, track: soundcloud_rs::Track) {
        self.inner.write().await.soundcloud_tracks.insert(id, track);
    }

    pub async fn get_continuation(&self, query: &str) -> Option<String> {
        self.inner
            .read()
            .await
            .search_continuations
            .get(&query.to_lowercase())
            .cloned()
    }

    pub async fn set_continuation(&self, query: &str, token: Option<String>) {
        let mut write = self.inner.write().await;
        let key = query.to_lowercase();
        if let Some(tok) = token {
            write.search_continuations.insert(key, tok);
        } else {
            write.search_continuations.remove(&key);
        }
    }

    pub async fn get_or_create_client(&self, proxy: Option<String>) -> Client {
        let read = self.inner.read().await;
        if let Some(client) = read.http_clients.get(&proxy) {
            return client.clone();
        }
        drop(read);

        let mut write = self.inner.write().await;
        if let Some(client) = write.http_clients.get(&proxy) {
            return client.clone();
        }
        let client = create_http_client_with_proxy(proxy.as_deref());
        write.http_clients.insert(proxy, client.clone());
        client
    }
}

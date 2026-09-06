use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::sync::RwLock;

use crate::{
    api::download_manager::DownloadManager, settings::SettingsManager, storage::MusicStorage,
};

#[derive(Default)]
struct CacheInner {
    youtube_ids: HashMap<i64, String>,
    streams: HashMap<i64, (String, Instant)>,
    soundcloud_tracks: HashMap<i64, soundcloud_rs::Track>,
    search_continuations: HashMap<String, String>,
    http_client: Option<(Option<String>, reqwest::Client)>,
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

    pub async fn get_stream(&self, id: i64) -> Option<String> {
        let read = self.inner.read().await;
        if let Some((url, instant)) = read.streams.get(&id)
            && instant.elapsed() < Duration::from_secs(3 * 3600)
        {
            return Some(url.clone());
        }
        None
    }

    pub async fn insert_stream(&self, id: i64, url: String) {
        self.inner
            .write()
            .await
            .streams
            .insert(id, (url, Instant::now()));
    }

    pub async fn remove_stream(&self, id: i64) {
        self.inner.write().await.streams.remove(&id);
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

    pub async fn get_or_create_client(&self, proxy: Option<String>) -> reqwest::Client {
        let mut inner = self.inner.write().await;
        if let Some((ref cached_proxy, ref client)) = inner.http_client
            && *cached_proxy == proxy
        {
            return client.clone();
        }
        let client = yt_audio_downloader::create_http_client_with_proxy(proxy.as_deref());
        inner.http_client = Some((proxy, client.clone()));
        client
    }
}

#[derive(Clone)]
pub struct AppState {
    pub client: Arc<soundcloud_rs::Client>,
    pub storage: Arc<RwLock<MusicStorage>>,
    pub download_manager: Arc<DownloadManager>,
    pub settings: SettingsManager,
    pub cache: AppCache,
}

impl AppState {
    pub fn new(
        client: soundcloud_rs::Client,
        storage: Arc<RwLock<MusicStorage>>,
        settings: SettingsManager,
    ) -> Self {
        Self {
            client: Arc::new(client),
            storage,
            download_manager: Arc::new(DownloadManager::default()),
            settings,
            cache: AppCache::default(),
        }
    }

    pub async fn http_client(&self) -> reqwest::Client {
        let settings = self.settings.read().await;
        let proxy = settings.network.get_proxy_url().map(String::from);
        drop(settings);
        self.cache.get_or_create_client(proxy).await
    }
}

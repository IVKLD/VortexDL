use std::{collections::HashMap, sync::Arc, time::Instant};

use tokio::sync::RwLock;

use crate::{
    api::download_manager::DownloadManager, settings::SettingsManager, storage::MusicStorage,
};

#[derive(Default, Clone)]
pub struct AppCache {
    pub youtube_ids: Arc<RwLock<HashMap<i64, String>>>,
    pub streams: Arc<RwLock<HashMap<i64, (String, Instant)>>>,
    pub soundcloud_tracks: Arc<RwLock<HashMap<i64, soundcloud_rs::Track>>>,
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
}

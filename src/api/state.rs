use std::{collections::HashMap, sync::Arc, time::Instant};

use tokio::sync::RwLock;

use crate::{
    api::download_manager::DownloadManager, settings::SettingsManager, storage::MusicStorage,
};

#[derive(Clone)]
pub struct AppState {
    pub client: Arc<soundcloud_rs::Client>,
    pub storage: Arc<RwLock<MusicStorage>>,
    pub download_manager: Arc<DownloadManager>,
    pub settings: SettingsManager,
    pub youtube_cache: Arc<RwLock<HashMap<i64, String>>>,
    pub stream_cache: Arc<RwLock<HashMap<i64, (String, Instant)>>>,
}

impl AppState {
    pub fn from_parts(
        client: soundcloud_rs::Client,
        storage: Arc<RwLock<MusicStorage>>,
        settings: SettingsManager,
    ) -> Self {
        Self {
            client: Arc::new(client),
            storage,
            download_manager: Arc::new(DownloadManager::default()),
            settings,
            youtube_cache: Arc::new(RwLock::new(HashMap::new())),
            stream_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

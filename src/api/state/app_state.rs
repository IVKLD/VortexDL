use std::sync::Arc;

use tokio::sync::RwLock;

use super::cache::AppCache;
use crate::{
    api::download_manager::DownloadManager, settings::SettingsManager, storage::MusicStorage,
};

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
        let proxy = self
            .settings
            .read()
            .await
            .network
            .get_proxy_url()
            .map(String::from);
        self.cache.get_or_create_client(proxy).await
    }
}

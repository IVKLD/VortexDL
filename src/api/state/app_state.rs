use std::sync::Arc;

use reqwest::Client as HttpClient;
use soundcloud_rs::Client as SoundcloudClient;
use tokio::sync::RwLock;

use super::cache::AppCache;
use crate::{
    download_manager::DownloadManager, downloader::Context, settings::SettingsManager,
    storage::MusicStorage,
};

#[derive(Clone)]
pub struct AppState {
    pub client: Arc<SoundcloudClient>,
    pub storage: Arc<RwLock<MusicStorage>>,
    pub download_manager: Arc<DownloadManager>,
    pub settings: SettingsManager,
    pub cache: AppCache,
}

impl AppState {
    pub fn new(
        client: SoundcloudClient,
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

    pub fn downloader_context(&self) -> Context {
        Context::new(
            self.storage.clone(),
            self.client.clone(),
            Some(self.download_manager.clone()),
            self.settings.clone(),
        )
    }

    pub async fn http_client(&self) -> HttpClient {
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

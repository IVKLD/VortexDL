use std::sync::Arc;

use reqwest::Client as HttpClient;
use soundcloud_rs::Client as SoundcloudClient;
use tokio::sync::RwLock;

use crate::{
    downloader::{DownloadContext, DownloadManager},
    library::Library,
    providers::cache::ProviderCache,
    settings::SettingsManager,
};

#[derive(Clone)]
pub struct AppState {
    pub soundcloud: Arc<SoundcloudClient>,
    pub library: Arc<RwLock<Library>>,
    pub download_manager: Arc<DownloadManager>,
    pub settings: SettingsManager,
    pub cache: ProviderCache,
}

impl AppState {
    pub fn new(
        soundcloud: SoundcloudClient,
        library: Arc<RwLock<Library>>,
        settings: SettingsManager,
    ) -> Self {
        Self {
            soundcloud: Arc::new(soundcloud),
            library,
            download_manager: Arc::new(DownloadManager::default()),
            settings,
            cache: ProviderCache::default(),
        }
    }

    pub fn downloader_context(&self) -> DownloadContext {
        DownloadContext::new(
            self.library.clone(),
            self.soundcloud.clone(),
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
        self.cache.get_or_create_client(proxy)
    }
}

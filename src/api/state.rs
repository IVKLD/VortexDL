use std::sync::Arc;

use tokio::sync::RwLock;

use crate::{
    api::download_manager::DownloadManager, settings::SettingsManager, storage::MusicStorage,
};

#[derive(Clone)]
pub struct AppState {
    pub client: Arc<soundcloud_rs::Client>,
    pub http: reqwest::Client,
    pub storage: Arc<RwLock<MusicStorage>>,
    pub download_manager: Arc<DownloadManager>,
    pub settings: SettingsManager,
}

impl AppState {
    pub fn from_parts(
        client: soundcloud_rs::Client,
        storage: Arc<RwLock<MusicStorage>>,
        settings: SettingsManager,
    ) -> Self {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client: Arc::new(client),
            http,
            storage,
            download_manager: Arc::new(DownloadManager::default()),
            settings,
        }
    }
}

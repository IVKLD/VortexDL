use std::sync::Arc;

use tokio::sync::RwLock;

use crate::{
    api::download_manager::DownloadManager,
    settings::{SettingsManager, UserSettings},
    storage::MusicStorage,
};

#[derive(Clone)]
pub struct AppState {
    pub client: Arc<soundcloud_rs::Client>,
    pub http: Arc<reqwest::Client>,
    pub storage: Arc<RwLock<MusicStorage>>,
    pub download_manager: Arc<DownloadManager>,
    pub settings: SettingsManager,
}

impl AppState {
    pub fn new(
        client: Arc<soundcloud_rs::Client>,
        storage: Arc<RwLock<MusicStorage>>,
        settings: UserSettings,
    ) -> Self {
        Self {
            client,
            http: Arc::new(reqwest::Client::new()),
            storage,
            download_manager: Arc::new(DownloadManager::default()),
            settings: SettingsManager::new(settings),
        }
    }
}

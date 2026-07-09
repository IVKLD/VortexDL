use std::sync::Arc;

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
        }
    }
}

use std::sync::Arc;

use tokio::sync::RwLock;

use crate::{
    api::download_manager::DownloadManager, database::settings::UserSettings, storage::MusicStorage,
};

#[derive(Clone)]
pub struct AppState {
    pub client: Arc<soundcloud_rs::Client>,
    pub http: Arc<reqwest::Client>,
    pub storage: Arc<RwLock<MusicStorage>>,
    pub download_manager: Arc<DownloadManager>,
    pub settings: Arc<RwLock<UserSettings>>,
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
            settings: Arc::new(RwLock::new(settings)),
        }
    }
}

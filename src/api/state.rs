use std::sync::Arc;

use soundcloud_rs::Client;
use tokio::sync::RwLock;

use crate::{
    api::download_manager::DownloadManager, database::settings::UserSettings, storage::MusicStorage,
};

#[derive(Clone)]
pub struct AppState {
    pub client: Arc<Client>,
    pub storage: Arc<RwLock<MusicStorage>>,
    pub download_manager: Arc<DownloadManager>,
    pub settings: Arc<RwLock<UserSettings>>,
}

impl AppState {
    pub fn new(client: Arc<Client>, output_dir: String, settings: UserSettings) -> Self {
        Self {
            client,
            storage: Arc::new(RwLock::new(MusicStorage::new(output_dir))),
            download_manager: Arc::new(DownloadManager::default()),
            settings: Arc::new(RwLock::new(settings)),
        }
    }
}

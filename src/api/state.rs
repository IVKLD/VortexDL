use std::sync::Arc;

use soundcloud_rs::Client;
use tokio::sync::RwLock;

use crate::{api::download_manager::DownloadManager, config::AppConfig, storage::MusicStorage};

#[derive(Clone)]
pub struct AppState {
    pub client: Arc<Client>,
    pub config: Arc<AppConfig>,
    pub storage: Arc<RwLock<MusicStorage>>,
    pub download_manager: Arc<DownloadManager>,
}

impl AppState {
    pub fn new(client: Arc<Client>, config: Arc<AppConfig>, output_dir: String) -> Self {
        Self {
            client,
            config,
            storage: Arc::new(RwLock::new(MusicStorage::new(output_dir))),
            download_manager: Arc::new(DownloadManager::default()),
        }
    }
}

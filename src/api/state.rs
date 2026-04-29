use std::sync::Arc;
use tokio::sync::RwLock;

use soundcloud_rs::Client;

use crate::config::AppConfig;
use crate::storage::MusicStorage;
use super::download_manager::DownloadManager;


#[derive(Clone)]
pub struct AppState {
    pub client: Arc<Client>,
    pub config: Arc<AppConfig>,
    pub output_dir: Arc<String>,
    pub storage: Arc<RwLock<MusicStorage>>,
    pub download_manager: Arc<DownloadManager>,
}

impl AppState {
    pub fn new(client: Arc<Client>, config: Arc<AppConfig>, output_dir: String) -> Self {
        Self {
            client,
            config,
            output_dir: Arc::new(output_dir),
            storage: Arc::new(RwLock::new(MusicStorage::default())),
            download_manager: Arc::new(DownloadManager::default()),
        }
    }
}

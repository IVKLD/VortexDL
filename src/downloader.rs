use std::sync::Arc;

use soundcloud_rs::Client;
use tokio::sync::RwLock;

use crate::{api::download_manager::DownloadManager, config::AppConfig, storage::MusicStorage};

mod core;
mod discovery;
mod dispatcher;
mod utils;

pub use dispatcher::dispatch_download;

pub(crate) struct Context {
    pub storage: Arc<RwLock<MusicStorage>>,
    pub client: Arc<Client>,
    pub config: Arc<AppConfig>,
    pub dm: Option<Arc<DownloadManager>>,
}

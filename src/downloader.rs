use std::sync::Arc;

use soundcloud_rs::Client;
use tokio::sync::RwLock;

use crate::{
    api::download_manager::DownloadManager, database::settings::UserSettings, storage::MusicStorage,
};

mod core;
mod discovery;
mod dispatcher;
mod utils;

pub use dispatcher::dispatch_download;

pub(crate) struct Context {
    pub storage: Arc<RwLock<MusicStorage>>,
    pub client: Arc<Client>,
    pub dm: Option<Arc<DownloadManager>>,
    pub settings: Arc<RwLock<UserSettings>>,
}

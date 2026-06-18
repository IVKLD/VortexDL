use std::sync::Arc;

use tokio::sync::RwLock;

use crate::{
    api::{download_manager::DownloadManager, state::AppState},
    settings::SettingsManager,
    storage::MusicStorage,
    types::discovery::AsUsername,
    utils::filename::clean_title,
};

#[derive(Clone)]
pub struct Context {
    pub storage: Arc<RwLock<MusicStorage>>,
    pub client: Arc<soundcloud_rs::Client>,
    pub http: reqwest::Client,
    pub dm: Option<Arc<DownloadManager>>,
    pub settings: SettingsManager,
}

impl Context {
    pub fn from_state(state: &AppState) -> Self {
        Self {
            storage: state.storage.clone(),
            client: state.client.clone(),
            http: state.http.clone(),
            dm: None,
            settings: state.settings.clone(),
        }
    }

    pub fn with_dm(mut self, dm: Arc<DownloadManager>) -> Self {
        self.dm = Some(dm);
        self
    }
}

#[derive(Clone, Debug)]
pub struct TrackDownload {
    pub id: i64,
    pub title: String,
    pub artist: String,
    pub artwork_url: Option<String>,
    pub position: Option<u32>,
}

impl TrackDownload {
    pub fn new<T: AsUsername>(
        id: i64,
        title: Option<&str>,
        user: Option<&T>,
        artwork_url: Option<String>,
        position: Option<u32>,
    ) -> Self {
        let artist = user
            .and_then(|u| u.username())
            .unwrap_or("Unknown")
            .to_string();
        let title = clean_title(title.unwrap_or("Unknown"));
        Self {
            id,
            title,
            artist,
            artwork_url,
            position,
        }
    }
}


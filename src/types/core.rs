use std::sync::Arc;
use std::fmt;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{
    api::{download_manager::DownloadManager, state::AppState},
    settings::SettingsManager,
    storage::MusicStorage,
    utils::filename::clean_title,
};

// --- Sync Mode ---

#[derive(ValueEnum, Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[value(rename_all = "lowercase")]
pub enum SyncMode {
    Silent,
    Full,
    Archive,
}

impl fmt::Display for SyncMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Silent => write!(f, "silent"),
            Self::Full => write!(f, "full"),
            Self::Archive => write!(f, "archive"),
        }
    }
}

// --- Downloader Context and Batch Types ---

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
    pub fn filename(&self) -> String {
        crate::utils::filename::clean_filename(&format!("{} - {}", self.artist, self.title))
    }

    pub fn path(&self, output_dir: impl AsRef<std::path::Path>) -> std::path::PathBuf {
        output_dir.as_ref().join(format!("{}.mp3", self.filename()))
    }

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

// --- Discovery / SoundCloud Scraper Types ---

#[derive(Serialize)]
pub struct ResolveQuery {
    pub url: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ResolveResponse {
    pub id: i64,
    pub kind: String,
}

#[derive(Deserialize, Debug)]
pub struct TrackLikesResponse {
    pub collection: Vec<LikeItem>,
    pub next_href: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct LikeItem {
    pub track: Option<TrackInfo>,
}

#[derive(Deserialize, Debug)]
pub struct TrackInfo {
    pub id: i64,
    pub title: String,
    pub artwork_url: Option<String>,
    pub user: Option<UserInfo>,
}

#[derive(Deserialize, Debug)]
pub struct UserInfo {
    pub username: String,
}

#[derive(Serialize)]
pub struct TrackLikesQuery {
    pub limit: u32,
    pub offset: Option<String>,
}

pub trait AsUsername {
    fn username(&self) -> Option<&str>;
}

impl AsUsername for UserInfo {
    fn username(&self) -> Option<&str> {
        Some(&self.username)
    }
}

impl AsUsername for soundcloud_rs::UserSummary {
    fn username(&self) -> Option<&str> {
        self.username.as_deref()
    }
}

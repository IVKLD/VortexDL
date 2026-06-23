use std::{fmt, sync::Arc};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use utoipa::ToSchema;

use crate::{
    api::{download_manager::DownloadManager, state::AppState},
    settings::SettingsManager,
    storage::MusicStorage,
    utils::filename::clean_title,
};

// --- Sync Mode ---

#[derive(ValueEnum, Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
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
pub struct DiscoveredMusicTrack {
    pub id: i64,
    pub title: String,
    pub artist: String,
    pub artwork_url: Option<String>,
    pub position: Option<u32>,
}

impl DiscoveredMusicTrack {
    pub fn new(
        id: i64,
        title: Option<&str>,
        artist: Option<&soundcloud_rs::UserSummary>,
        artwork_url: Option<String>,
    ) -> Self {
        let artist = artist
            .and_then(|u| u.username.as_deref())
            .unwrap_or("Unknown")
            .to_string();
        let title = clean_title(title.unwrap_or("Unknown"));
        Self {
            id,
            title,
            artist,
            artwork_url,
            position: None,
        }
    }

    pub fn from_track(track: soundcloud_rs::Track) -> Option<Self> {
        let id = track.id?;
        Some(Self::new(
            id,
            track.title.as_deref(),
            track.user.as_ref(),
            track.artwork_url,
        ))
    }

    pub fn with_position(mut self, position: Option<u32>) -> Self {
        self.position = position;
        self
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
pub struct MusicTrackLikesResponse {
    pub collection: Vec<LikeItem>,
    pub next_href: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct LikeItem {
    pub track: Option<soundcloud_rs::Track>,
}

#[derive(Serialize)]
pub struct MusicTrackLikesQuery {
    pub limit: u32,
    pub offset: Option<String>,
}

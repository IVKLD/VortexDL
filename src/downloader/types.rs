use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use soundcloud_rs::Client as SoundcloudClient;
use tokio::sync::RwLock;
use url::Url;
use utoipa::ToSchema;

use super::manager::DownloadManager;
use crate::{
    domain::{AudioFormat, DiscoveredTrack, clean_filename},
    library::Library,
    settings::SettingsManager,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum DownloadStatus {
    Queued,
    Downloading,
    Finished,
    Failed,
    Canceled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum MessageLevel {
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DownloadItem {
    pub id: i64,
    pub title: String,
    pub artist: String,
    #[schema(value_type = Option<String>)]
    pub artwork_url: Option<Url>,
    #[schema(value_type = Option<String>)]
    pub source_url: Option<Url>,
    pub status: DownloadStatus,
    pub format: Option<AudioFormat>,
    pub created_at: Option<u64>,
    pub progress: Option<f64>,
    pub size: Option<u64>,
    pub error: Option<String>,
}

impl DownloadItem {
    pub fn is_active(&self) -> bool {
        matches!(
            self.status,
            DownloadStatus::Queued | DownloadStatus::Downloading
        )
    }
}

impl From<DiscoveredTrack> for DownloadItem {
    fn from(task: DiscoveredTrack) -> Self {
        Self {
            id: task.id,
            title: task.title,
            artist: task.artist,
            artwork_url: task.artwork_url,
            source_url: task.permalink_url,
            status: DownloadStatus::Queued,
            format: None,
            created_at: None,
            progress: None,
            size: None,
            error: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ServerEvent {
    TrackUpdate {
        item: Box<DownloadItem>,
    },
    SyncFinished {
        url: Option<String>,
    },
    SyncStarted {
        url: String,
    },
    Error {
        message: String,
    },
    Message {
        message: String,
        level: MessageLevel,
    },
}

#[derive(Clone)]
pub struct DownloadContext {
    pub library: Arc<RwLock<Library>>,
    pub soundcloud: Arc<SoundcloudClient>,
    pub manager: Option<Arc<DownloadManager>>,
    pub settings: SettingsManager,
}

impl DownloadContext {
    pub fn new(
        library: Arc<RwLock<Library>>,
        soundcloud: Arc<SoundcloudClient>,
        manager: Option<Arc<DownloadManager>>,
        settings: SettingsManager,
    ) -> Self {
        Self {
            library,
            soundcloud,
            manager,
            settings,
        }
    }
}

pub struct DownloadTask {
    pub track: DiscoveredTrack,
    pub file_path: PathBuf,
}

impl DownloadTask {
    pub fn display_name(&self) -> String {
        format!("{} - {}", self.track.artist, self.track.title)
    }

    pub fn new(track: &DiscoveredTrack, naming_template: &str, output_dir: &Path) -> Self {
        let formatted = naming_template
            .replace("{artist}", &track.artist)
            .replace("{title}", &track.title);
        let filename = format!("{}.mp3", clean_filename(&formatted));

        Self {
            track: track.clone(),
            file_path: output_dir.join(filename),
        }
    }
}

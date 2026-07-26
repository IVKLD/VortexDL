use serde::{Deserialize, Serialize};
use url::Url;
use utoipa::ToSchema;

use crate::{api::types::AudioFormat, types::DiscoveredMusicTrack};

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
pub struct DownloadTrackDetails {
    pub title: String,
    pub artist: String,
    #[schema(value_type = Option<String>)]
    pub artwork_url: Option<Url>,
    #[schema(value_type = Option<String>)]
    pub source_url: Option<Url>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DownloadItem {
    pub id: i64,
    #[serde(flatten)]
    pub details: DownloadTrackDetails,
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

impl From<DiscoveredMusicTrack> for DownloadItem {
    fn from(task: DiscoveredMusicTrack) -> Self {
        Self {
            id: task.id,
            details: DownloadTrackDetails {
                title: task.title,
                artist: task.artist,
                artwork_url: task.artwork_url,
                source_url: task.permalink_url,
            },
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
    TrackUpdate { item: Box<DownloadItem> },
    SyncFinished { url: Option<String> },
    SyncStarted { url: String },
    Error { message: String },
    Message { message: String, level: MessageLevel },
}

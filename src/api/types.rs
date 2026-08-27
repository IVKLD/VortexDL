use serde::{Deserialize, Serialize};
use url::Url;
use utoipa::ToSchema;

use crate::storage::LocalMusicTrack;

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DownloadRequest {
    #[schema(value_type = String)]
    pub url: Url,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ApiStatus {
    Ok,
    Queued,
    Error,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DownloadStartResponse {
    pub status: ApiStatus,
    pub message: String,
}

pub use crate::types::AudioFormat;

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MusicTrackRecord {
    pub id: i64,
    pub artist: String,
    pub title: String,
    pub format: AudioFormat,
    pub artwork_url: Option<String>,
    pub source_url: Option<String>,
    pub created_at: u64,
    pub size: u64,
    pub archived: bool,
}

impl MusicTrackRecord {
    pub fn from_local_track(id: i64, data: &LocalMusicTrack) -> Self {
        Self {
            id,
            artist: data.metadata.artist.clone(),
            title: data.metadata.title.clone(),
            artwork_url: data.metadata.artwork_url.clone(),
            source_url: data.metadata.source_url.clone(),
            format: AudioFormat::from_path(&data.path),
            created_at: data.created_at,
            size: data.size,
            archived: data.is_archived(),
        }
    }
}

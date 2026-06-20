use serde::{Deserialize, Serialize};

use crate::storage::LocalTrack;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadRequest {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ApiStatus {
    Ok,
    Queued,
    Error,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadStartResponse {
    pub status: ApiStatus,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AudioFormat {
    Mp3,
    Flac,
    Wav,
    Unknown,
}

impl AudioFormat {
    pub fn from_path(path: impl AsRef<std::path::Path>) -> Self {
        path.as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| match ext.to_lowercase().as_str() {
                "mp3" => Self::Mp3,
                "flac" => Self::Flac,
                "wav" => Self::Wav,
                _ => Self::Unknown,
            })
            .unwrap_or(Self::Unknown)
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackRecord {
    pub id: i64,
    pub artist: String,
    pub title: String,
    pub format: AudioFormat,
    pub artwork_url: Option<String>,
    pub source_url: Option<String>,
    pub created_at: u64,
    pub size: u64,
    pub position: u32,
    pub archived: bool,
}

impl TrackRecord {
    pub fn from_local_track(id: i64, data: &LocalTrack) -> Self {
        Self {
            id,
            artist: data.artist.clone(),
            title: data.title.clone(),
            format: AudioFormat::from_path(&data.path),
            artwork_url: data.artwork_url.clone(),
            source_url: data.source_url.clone(),
            created_at: data.created_at,
            size: data.size,
            position: data.position.unwrap_or(u32::MAX),
            archived: data.is_archived(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {
    pub status: ApiStatus,
}

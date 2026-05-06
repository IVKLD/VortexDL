use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct DownloadRequest {
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct ActionStatus {
    pub status: &'static str,
    pub message: String,
}

#[derive(Eq, PartialEq)]
pub enum TrackExtension {
    MP3,
    FLAC,
    WAV,
    Unknown
}

pub const KNOWN_EXTENSIONS: [TrackExtension; 3] = [
    TrackExtension::MP3,
    TrackExtension::FLAC,
    TrackExtension::WAV,
];

#[derive(Debug, Serialize)]
pub struct TrackRecord {
    pub id: u32,
    pub filename: String,
    pub album: String,
    pub format: String,
    pub artwork_url: Option<String>,
    pub source_url: Option<String>,
    pub created_at: u64,
}

#[derive(Debug, Deserialize)]
pub struct DeleteQuery {
    pub id: u32,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
}

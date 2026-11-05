use serde::{Deserialize, Serialize};
use soundcloud_rs::Track as SoundCloudTrack;
use utoipa::ToSchema;
use yt_audio_downloader::VideoMetadata;

use crate::domain;

#[derive(Debug, Deserialize, Serialize, ToSchema, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum SearchProviderParam {
    Youtube,
    Soundcloud,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum SearchDurationFilter {
    Any,
    Short,
    Medium,
    Long,
    Epic,
}

impl SearchDurationFilter {
    pub fn matches(self, duration_seconds: u64) -> bool {
        match self {
            Self::Short => duration_seconds < 120,
            Self::Medium => (120..=600).contains(&duration_seconds),
            Self::Long => (601..=1800).contains(&duration_seconds),
            Self::Epic => duration_seconds > 1800,
            Self::Any => true,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub provider: Option<SearchProviderParam>,
    pub duration: Option<SearchDurationFilter>,
}

#[derive(Debug, Serialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SearchTrackItem {
    pub id: i64,
    pub title: String,
    pub artist: String,
    pub artwork_url: Option<String>,
    pub duration: Option<i64>,
    pub playback_count: Option<i64>,
    pub permalink_url: Option<String>,
    pub genre: Option<String>,
}

impl SearchTrackItem {
    pub fn from_youtube_meta(meta: &VideoMetadata) -> (i64, String, Self) {
        let id = yt_audio_downloader::youtube_id_to_i64(&meta.id);
        let permalink_url = format!("https://www.youtube.com/watch?v={}", meta.id);
        let (artist, title) = domain::parse_track_metadata(&meta.title, &meta.author);
        let item = Self {
            id,
            title,
            artist,
            artwork_url: meta.thumbnail_url.clone(),
            duration: Some((meta.duration_seconds * 1000) as i64),
            playback_count: Some(meta.view_count as i64),
            permalink_url: Some(permalink_url),
            genre: None,
        };
        (id, meta.id.clone(), item)
    }

    pub fn from_soundcloud_track(id: i64, track: SoundCloudTrack) -> Self {
        let artist = track
            .user
            .as_ref()
            .and_then(|u| u.username.clone())
            .unwrap_or_else(|| "Unknown".to_string());
        let artwork_url = track.artwork_url_with_size("t200x200");

        Self {
            id,
            title: track.title.unwrap_or_default(),
            artist,
            artwork_url,
            duration: track.duration,
            playback_count: track.playback_count,
            permalink_url: track.permalink_url,
            genre: track.genre,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    pub tracks: Vec<SearchTrackItem>,
    pub has_more: bool,
}

impl SearchResponse {
    pub fn empty() -> Self {
        Self {
            tracks: Vec::new(),
            has_more: false,
        }
    }
}

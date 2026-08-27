use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    storage::metadata::detect_platform_str,
    utils::{filename::parse_track_metadata, soundcloud::resize_artwork_url},
};

#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, utoipa::ToSchema)]
pub struct TrackMetadata {
    pub id: i64,
    pub artist: String,
    pub title: String,
    pub artwork_url: Option<String>,
    pub source_url: Option<String>,
    pub platform: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DiscoveredMusicTrack {
    pub id: i64,
    pub title: String,
    pub artist: String,
    #[schema(value_type = Option<String>)]
    pub artwork_url: Option<Url>,
    #[schema(value_type = Option<String>)]
    pub permalink_url: Option<Url>,
    pub duration_ms: Option<u64>,
}

impl DiscoveredMusicTrack {
    pub fn new(
        id: i64,
        title: Option<&str>,
        artist: Option<&soundcloud_rs::UserSummary>,
        artwork_url: Option<Url>,
        permalink_url: Option<Url>,
        duration_ms: Option<u64>,
    ) -> Self {
        let uploader = artist
            .and_then(|u| u.username.as_deref())
            .unwrap_or("Unknown");
        let raw_title = title.unwrap_or("Unknown");

        let (artist, title) = parse_track_metadata(raw_title, uploader);

        let artwork_url = artwork_url.map(|url| resize_artwork_url(url, "-t1080x1080"));

        Self {
            id,
            title,
            artist,
            artwork_url,
            permalink_url,
            duration_ms,
        }
    }

    pub fn from_track(track: soundcloud_rs::Track) -> Option<Self> {
        let id = track.id?;
        let artwork_url = track.artwork_url.and_then(|s| Url::parse(&s).ok());
        let permalink_url = track.permalink_url.and_then(|s| Url::parse(&s).ok());
        let duration_ms = track.duration.map(|d| d as u64);
        Some(Self::new(
            id,
            track.title.as_deref(),
            track.user.as_ref(),
            artwork_url,
            permalink_url,
            duration_ms,
        ))
    }

    pub fn to_metadata(&self, source_url: Option<String>) -> TrackMetadata {
        let platform = detect_platform_str(source_url.as_deref()).to_string();

        TrackMetadata {
            id: self.id,
            artist: self.artist.clone(),
            title: self.title.clone(),
            artwork_url: self.artwork_url.as_ref().map(|u| u.as_str().to_string()),
            source_url,
            platform,
        }
    }
}

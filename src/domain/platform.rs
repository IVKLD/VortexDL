use std::fmt::{Display, Formatter, Result as FmtResult};

use serde::{Deserialize, Serialize};
use soundcloud_rs::is_soundcloud_url;
use utoipa::ToSchema;
use yt_audio_downloader::is_youtube_url;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum SourcePlatform {
    SoundCloud,
    YouTube,
    Local,
}

impl SourcePlatform {
    pub fn detect_from_url_str(url: &str) -> Self {
        if is_youtube_url(url) {
            Self::YouTube
        } else if is_soundcloud_url(url) {
            Self::SoundCloud
        } else {
            Self::Local
        }
    }

    pub fn detect_from_optional_url(url: Option<&str>) -> Self {
        url.map(Self::detect_from_url_str).unwrap_or(Self::Local)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SoundCloud => "soundcloud",
            Self::YouTube => "youtube",
            Self::Local => "local",
        }
    }
}

impl Display for SourcePlatform {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.as_str())
    }
}

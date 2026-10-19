use serde::{Deserialize, Serialize};
use url::Url;
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum SourcePlatform {
    SoundCloud,
    YouTube,
    Local,
}

impl SourcePlatform {
    pub fn is_soundcloud_url(url: &str) -> bool {
        if let Ok(parsed) = Url::parse(url)
            && let Some(host) = parsed.host_str()
        {
            let host = host.to_lowercase();
            return host == "soundcloud.com"
                || host.ends_with(".soundcloud.com")
                || host == "on.soundcloud.com"
                || host == "soundcloud.cloud"
                || host.ends_with(".soundcloud.cloud");
        }
        let lower = url.to_lowercase();
        lower.contains("soundcloud.com")
            || lower.contains("on.soundcloud.com")
            || lower.contains("soundcloud.cloud")
    }

    pub fn detect_from_url_str(url: &str) -> Self {
        if yt_audio_downloader::is_youtube_url(url) {
            Self::YouTube
        } else if Self::is_soundcloud_url(url) {
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

impl std::fmt::Display for SourcePlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_url_detection() {
        assert!(SourcePlatform::is_soundcloud_url(
            "https://soundcloud.com/artist/track"
        ));
        assert!(SourcePlatform::is_soundcloud_url(
            "https://m.soundcloud.com/artist/track"
        ));
        assert!(SourcePlatform::is_soundcloud_url(
            "https://on.soundcloud.com/abc123xyz"
        ));
        assert!(SourcePlatform::is_soundcloud_url(
            "https://playback.media-streaming.soundcloud.cloud/y2BhMavepv0e/aac_160k/playlist.m3u8"
        ));
        assert!(!SourcePlatform::is_soundcloud_url(
            "https://youtube.com/watch?v=123"
        ));

        assert_eq!(
            SourcePlatform::detect_from_url_str("https://soundcloud.com/artist/track"),
            SourcePlatform::SoundCloud
        );
        assert_eq!(
            SourcePlatform::detect_from_url_str("https://youtu.be/dQw4w9WgXcQ"),
            SourcePlatform::YouTube
        );
        assert_eq!(
            SourcePlatform::detect_from_url_str("https://example.com/audio.mp3"),
            SourcePlatform::Local
        );
        assert_eq!(
            SourcePlatform::detect_from_optional_url(None),
            SourcePlatform::Local
        );
    }
}

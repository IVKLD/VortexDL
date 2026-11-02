use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("SoundCloud error: {0}")]
    SoundCloud(#[from] soundcloud_rs::Error),

    #[error("YouTube error: {0}")]
    YouTube(#[from] yt_audio_downloader::YoutubeAudioError),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid URL: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("Upstream returned HTTP {0}")]
    UpstreamStatus(u16),

    #[error("Unsupported music URL: '{0}'. Supported platforms: SoundCloud and YouTube.")]
    UnsupportedPlatform(String),

    #[error("{0}")]
    InvalidStream(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl ProviderError {
    pub fn is_expired_status(&self) -> bool {
        matches!(self, Self::UpstreamStatus(401 | 403 | 404 | 410))
    }
}

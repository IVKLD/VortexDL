pub mod id;
pub mod innertube;
pub mod strategy;
pub mod ytdlp;

pub use id::extract_video_id;
pub use innertube::InnertubeExtractor;
pub use strategy::MediaExtractor;
pub use ytdlp::YtDlpExtractor;

use crate::error::Result;
use crate::models::{AudioStreamInfo, ExtractedMedia, VideoMetadata};
use reqwest::Client;
use std::sync::Arc;

pub struct YoutubeExtractor {
    primary: Arc<dyn MediaExtractor>,
    fallback: Arc<dyn MediaExtractor>,
}

impl YoutubeExtractor {
    pub fn new(client: Client) -> Self {
        Self {
            primary: Arc::new(InnertubeExtractor::new(client)),
            fallback: Arc::new(YtDlpExtractor::new()),
        }
    }

    pub async fn extract_media(&self, target: &str) -> Result<ExtractedMedia> {
        match self.primary.extract(target).await {
            Ok(media) => Ok(media),
            Err(_) => self.fallback.extract(target).await,
        }
    }

    pub async fn fetch_natively(&self, video_id: &str) -> Result<(VideoMetadata, Vec<AudioStreamInfo>)> {
        let media = self.primary.extract(video_id).await?;
        Ok((media.metadata, media.streams))
    }

    pub async fn fetch_fallback(video_url: &str) -> Result<(VideoMetadata, String)> {
        let extractor = YtDlpExtractor::new();
        let media = extractor.extract(video_url).await?;
        let url = media.best_stream().map(|s| s.url.clone()).unwrap_or_default();
        Ok((media.metadata, url))
    }
}

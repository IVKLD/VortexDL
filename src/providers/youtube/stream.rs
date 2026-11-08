use yt_audio_downloader::{AudioByteStream, AudioStreamResponse};

use crate::providers::{ProviderError, youtube::YouTubeProvider};

impl YouTubeProvider {
    /// Stream audio bytes for a resolved stream info within a byte range.
    pub async fn stream(
        &self,
        info: &AudioStreamResponse,
        start_offset: u64,
        end_offset: Option<u64>,
    ) -> Result<AudioByteStream, ProviderError> {
        let stream = self
            .client
            .stream_from_info(info, start_offset, end_offset)
            .await?;
        Ok(stream)
    }
}

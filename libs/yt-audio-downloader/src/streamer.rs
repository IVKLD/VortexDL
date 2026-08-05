use crate::error::{Result, YoutubeAudioError};
use crate::extractor::YoutubeExtractor;
use crate::http::create_http_client;
use crate::models::{AudioStreamResponse, VideoMetadata};
use bytes::Bytes;
use futures_util::Stream;
use reqwest::Client;

pub struct AudioStreamer {
    client: Client,
}

impl Default for AudioStreamer {
    fn default() -> Self {
        Self {
            client: create_http_client(),
        }
    }
}

impl AudioStreamer {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn get_stream_response(&self, url_or_id: &str) -> Result<AudioStreamResponse> {
        let extractor = YoutubeExtractor::new(self.client.clone());
        let media = extractor.extract_media(url_or_id).await?;
        let best = media.best_stream().cloned().ok_or(YoutubeAudioError::NoAudioStreamFound)?;
        let metadata = media.metadata;

        Ok(AudioStreamResponse {
            metadata,
            stream_url: best.url.clone(),
            mime_type: best.mime_type.clone(),
            content_length: best.content_length,
            stream_info: best,
        })
    }

    pub async fn stream_bytes(
        &self,
        url_or_id: &str,
    ) -> Result<(VideoMetadata, impl Stream<Item = reqwest::Result<Bytes>>)> {
        let response_info = self.get_stream_response(url_or_id).await?;
        let http_response = self.client.get(&response_info.stream_url).send().await?;
        Ok((response_info.metadata, http_response.bytes_stream()))
    }
}

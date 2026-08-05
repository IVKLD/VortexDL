use super::id::extract_video_id;
use super::strategy::MediaExtractor;
use crate::error::{Result, YoutubeAudioError};
use crate::models::{AudioStreamInfo, ExtractedMedia, VideoMetadata};
use reqwest::Client;
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;

pub struct InnertubeExtractor {
    client: Client,
}

impl InnertubeExtractor {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

impl MediaExtractor for InnertubeExtractor {
    fn extract<'a>(
        &'a self,
        target: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<ExtractedMedia>> + Send + 'a>> {
        Box::pin(async move {
            let video_id = extract_video_id(target)?;
            let url = "https://www.youtube.com/youtubei/v1/player";
            let payload = serde_json::json!({
                "videoId": video_id,
                "context": {
                    "client": {
                        "clientName": "ANDROID",
                        "clientVersion": "19.09.37",
                        "hl": "en",
                        "gl": "US"
                    }
                }
            });

            let json: Value = self.client.post(url).json(&payload).send().await?.json().await?;

            let details = json
                .get("videoDetails")
                .ok_or_else(|| YoutubeAudioError::DownloadFailed("Missing videoDetails".into()))?;

            let metadata = VideoMetadata {
                id: video_id,
                title: details.get("title").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
                author: details.get("author").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
                duration_seconds: details
                    .get("lengthSeconds")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
                view_count: details
                    .get("viewCount")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
                thumbnail_url: details
                    .get("thumbnail")
                    .and_then(|t| t.get("thumbnails"))
                    .and_then(|arr| arr.as_array())
                    .and_then(|arr| arr.last())
                    .and_then(|item| item.get("url"))
                    .and_then(|u| u.as_str())
                    .map(|s| s.to_string()),
                description: details.get("shortDescription").and_then(|v| v.as_str()).map(|s| s.to_string()),
            };

            let mut streams = Vec::new();
            if let Some(formats) = json
                .get("streamingData")
                .and_then(|s| s.get("adaptiveFormats"))
                .and_then(|v| v.as_array())
            {
                for fmt in formats {
                    let mime = fmt.get("mimeType").and_then(|v| v.as_str()).unwrap_or("");
                    if mime.starts_with("audio/") {
                        if let Some(direct_url) = fmt.get("url").and_then(|v| v.as_str()) {
                            let container = if mime.contains("webm") {
                                "webm"
                            } else if mime.contains("mp4") {
                                "m4a"
                            } else {
                                "audio"
                            };

                            let codec = if mime.contains("opus") {
                                "opus"
                            } else if mime.contains("mp4a") {
                                "aac"
                            } else {
                                "unknown"
                            };

                            streams.push(AudioStreamInfo {
                                url: direct_url.to_string(),
                                mime_type: mime.to_string(),
                                bitrate: fmt.get("bitrate").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                                sample_rate: fmt
                                    .get("audioSampleRate")
                                    .and_then(|v| v.as_str())
                                    .and_then(|s| s.parse().ok()),
                                content_length: fmt
                                    .get("contentLength")
                                    .and_then(|v| v.as_str())
                                    .and_then(|s| s.parse().ok()),
                                container: container.to_string(),
                                audio_codec: codec.to_string(),
                            });
                        }
                    }
                }
            }

            streams.sort_by(|a, b| b.bitrate.cmp(&a.bitrate));

            if streams.is_empty() {
                return Err(YoutubeAudioError::NoAudioStreamFound);
            }

            Ok(ExtractedMedia { metadata, streams })
        })
    }
}

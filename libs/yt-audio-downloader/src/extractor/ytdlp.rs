use super::strategy::MediaExtractor;
use crate::error::{Result, YoutubeAudioError};
use crate::models::{AudioStreamInfo, ExtractedMedia, VideoMetadata};
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;
use tokio::process::Command;

pub struct YtDlpExtractor;

impl YtDlpExtractor {
    pub fn new() -> Self {
        Self
    }
}

impl MediaExtractor for YtDlpExtractor {
    fn extract<'a>(
        &'a self,
        target: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<ExtractedMedia>> + Send + 'a>> {
        Box::pin(async move {
            let output = Command::new("yt-dlp")
                .args(["-j", "-f", "bestaudio/best", "--no-playlist", "--no-warnings", target])
                .output()
                .await
                .map_err(|e| {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        YoutubeAudioError::YtDlpNotFound
                    } else {
                        YoutubeAudioError::Io(e)
                    }
                })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                return Err(YoutubeAudioError::YtDlpFailed {
                    status: output.status.code(),
                    stderr,
                });
            }

            let json: Value = serde_json::from_slice(&output.stdout)?;

            let metadata = VideoMetadata {
                id: json.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                title: json.get("title").and_then(|v| v.as_str()).unwrap_or("Audio").to_string(),
                author: json
                    .get("uploader")
                    .or_else(|| json.get("channel"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string(),
                duration_seconds: json
                    .get("duration")
                    .and_then(|v| v.as_f64())
                    .map(|f| f as u64)
                    .unwrap_or(0),
                view_count: json.get("view_count").and_then(|v| v.as_u64()).unwrap_or(0),
                thumbnail_url: json.get("thumbnail").and_then(|v| v.as_str()).map(|s| s.to_string()),
                description: json.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
            };

            let stream_url = json
                .get("url")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default();

            if stream_url.is_empty() {
                return Err(YoutubeAudioError::NoAudioStreamFound);
            }

            let stream = AudioStreamInfo {
                url: stream_url,
                mime_type: "audio/webm".into(),
                bitrate: 0,
                sample_rate: None,
                content_length: None,
                container: "webm".into(),
                audio_codec: "unknown".into(),
            };

            Ok(ExtractedMedia {
                metadata,
                streams: vec![stream],
            })
        })
    }
}

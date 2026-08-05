use crate::converter::{sanitize_filename, AudioConverter};
use crate::error::{Result, YoutubeAudioError};
use crate::extractor::{extract_video_id, YoutubeExtractor};
use crate::http::create_http_client;
use crate::models::{AudioFormat, AudioQuality, AudioStreamResponse, DownloadedAudio, VideoMetadata};
use crate::progress::{ProgressEvent, ProgressHandler};
use crate::streamer::AudioStreamer;
use bytes::Bytes;
use futures_util::{Stream, StreamExt};
use reqwest::Client;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

pub struct YoutubeAudioDownloader {
    client: Client,
    output_dir: PathBuf,
    format: AudioFormat,
    quality: AudioQuality,
    progress_handler: Option<ProgressHandler>,
    prefer_yt_dlp: bool,
    embed_metadata: bool,
}

impl Default for YoutubeAudioDownloader {
    fn default() -> Self {
        Self {
            client: create_http_client(),
            output_dir: PathBuf::from("downloads"),
            format: AudioFormat::Mp3,
            quality: AudioQuality::Best,
            progress_handler: None,
            prefer_yt_dlp: false,
            embed_metadata: true,
        }
    }
}

impl YoutubeAudioDownloader {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn output_dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.output_dir = dir.as_ref().to_path_buf();
        self
    }

    pub fn format(mut self, format: AudioFormat) -> Self {
        self.format = format;
        self
    }

    pub fn quality(mut self, quality: AudioQuality) -> Self {
        self.quality = quality;
        self
    }

    pub fn on_progress<F>(mut self, handler: F) -> Self
    where
        F: Fn(ProgressEvent) + Send + Sync + 'static,
    {
        self.progress_handler = Some(Arc::new(handler));
        self
    }

    pub fn prefer_yt_dlp(mut self, prefer: bool) -> Self {
        self.prefer_yt_dlp = prefer;
        self
    }

    pub fn embed_metadata(mut self, embed: bool) -> Self {
        self.embed_metadata = embed;
        self
    }

    fn emit_progress(&self, event: ProgressEvent) {
        if let Some(ref handler) = self.progress_handler {
            handler(event);
        }
    }

    pub async fn fetch_metadata(&self, url_or_id: &str) -> Result<VideoMetadata> {
        let extractor = YoutubeExtractor::new(self.client.clone());
        let media = extractor.extract_media(url_or_id).await?;
        Ok(media.metadata)
    }

    pub async fn get_stream(&self, url_or_id: &str) -> Result<AudioStreamResponse> {
        let streamer = AudioStreamer::new(self.client.clone());
        streamer.get_stream_response(url_or_id).await
    }

    pub async fn stream_bytes(
        &self,
        url_or_id: &str,
    ) -> Result<(VideoMetadata, impl Stream<Item = reqwest::Result<Bytes>>)> {
        let streamer = AudioStreamer::new(self.client.clone());
        streamer.stream_bytes(url_or_id).await
    }

    pub async fn download(&self, url_or_id: &str) -> Result<DownloadedAudio> {
        let video_id = extract_video_id(url_or_id)?;
        self.emit_progress(ProgressEvent::Initializing {
            video_id: video_id.clone(),
        });

        tokio::fs::create_dir_all(&self.output_dir).await?;

        if self.prefer_yt_dlp {
            return self.download_with_ytdlp(url_or_id).await;
        }

        let extractor = YoutubeExtractor::new(self.client.clone());
        match extractor.extract_media(&video_id).await {
            Ok(media) => {
                let best_stream = match media.best_stream().cloned() {
                    Some(s) => s,
                    None => return self.download_with_ytdlp(url_or_id).await,
                };
                let metadata = media.metadata;

                self.emit_progress(ProgressEvent::MetadataFetched {
                    title: metadata.title.clone(),
                    author: metadata.author.clone(),
                });

                let temp_path = self.output_dir.join(format!("temp_{}.{}", video_id, best_stream.container));

                if self.download_stream_to_file(&best_stream.url, &temp_path).await.is_err() {
                    let _ = tokio::fs::remove_file(&temp_path).await;
                    return self.download_with_ytdlp(url_or_id).await;
                }

                self.emit_progress(ProgressEvent::Converting {
                    target_format: self.format.extension().to_string(),
                });

                let meta_param = if self.embed_metadata { Some(&metadata) } else { None };
                let final_path = match AudioConverter::convert(
                    &temp_path,
                    &self.output_dir,
                    &metadata.title,
                    self.format,
                    self.quality,
                    meta_param,
                )
                .await
                {
                    Ok(p) => p,
                    Err(_) => {
                        let _ = tokio::fs::remove_file(&temp_path).await;
                        return self.download_with_ytdlp(url_or_id).await;
                    }
                };

                let _ = tokio::fs::remove_file(&temp_path).await;
                let file_size = tokio::fs::metadata(&final_path).await?.len();

                self.emit_progress(ProgressEvent::Finished {
                    output_path: final_path.clone(),
                    total_bytes: file_size,
                });

                Ok(DownloadedAudio {
                    file_path: final_path,
                    metadata,
                    format: self.format,
                    file_size_bytes: file_size,
                })
            }
            _ => self.download_with_ytdlp(url_or_id).await,
        }
    }

    async fn download_stream_to_file(&self, stream_url: &str, output_path: &Path) -> Result<()> {
        let res = self.client.get(stream_url).send().await?;
        let total_size = res.content_length();

        let mut file = File::create(output_path).await?;
        let mut downloaded: u64 = 0;
        let mut stream = res.bytes_stream();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            file.write_all(&chunk).await?;

            downloaded += chunk.len() as u64;
            let percentage = total_size.map(|total| (downloaded as f32 / total as f32) * 100.0);

            self.emit_progress(ProgressEvent::Downloading {
                bytes_downloaded: downloaded,
                total_bytes: total_size,
                percentage,
            });
        }

        file.flush().await?;
        Ok(())
    }

    pub async fn download_with_ytdlp(&self, url_or_id: &str) -> Result<DownloadedAudio> {
        let (metadata, _) = YoutubeExtractor::fetch_fallback(url_or_id).await?;

        self.emit_progress(ProgressEvent::MetadataFetched {
            title: metadata.title.clone(),
            author: metadata.author.clone(),
        });

        let sanitized_title = sanitize_filename(&metadata.title);
        let output_template = self.output_dir.join(format!("{}.%(ext)s", sanitized_title));

        let audio_format_arg = match self.format {
            AudioFormat::Best => "best",
            fmt => fmt.extension(),
        };

        let mut cmd = Command::new("yt-dlp");
        cmd.args([
            "-x",
            "--audio-format",
            audio_format_arg,
            "--audio-quality",
            self.quality.bitrate_kbps(),
            "-o",
            output_template.to_str().unwrap_or_default(),
            "--no-playlist",
            "--no-warnings",
        ]);

        if self.embed_metadata {
            cmd.arg("--add-metadata");
        }

        cmd.arg(url_or_id);

        self.emit_progress(ProgressEvent::Downloading {
            bytes_downloaded: 0,
            total_bytes: None,
            percentage: None,
        });

        let output = cmd.output().await.map_err(|e| {
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

        let expected_final_path = self
            .output_dir
            .join(format!("{}.{}", sanitized_title, self.format.extension()));

        let final_path = if expected_final_path.exists() {
            expected_final_path
        } else {
            let mut entries = tokio::fs::read_dir(&self.output_dir).await?;
            let mut found = None;

            while let Ok(Some(entry)) = entries.next_entry().await {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with(&sanitized_title) {
                    found = Some(entry.path());
                    break;
                }
            }

            found.unwrap_or(expected_final_path)
        };

        let file_size = tokio::fs::metadata(&final_path)
            .await
            .map(|m| m.len())
            .unwrap_or(0);

        self.emit_progress(ProgressEvent::Finished {
            output_path: final_path.clone(),
            total_bytes: file_size,
        });

        Ok(DownloadedAudio {
            file_path: final_path,
            metadata,
            format: self.format,
            file_size_bytes: file_size,
        })
    }
}

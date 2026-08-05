pub mod converter;
pub mod downloader;
pub mod error;
pub mod extractor;
pub mod http;
pub mod models;
pub mod progress;
pub mod streamer;

pub use downloader::YoutubeAudioDownloader;
pub use error::{Result, YoutubeAudioError};
pub use http::create_http_client;
pub use models::{
    AudioFormat, AudioQuality, AudioStreamInfo, AudioStreamResponse, DownloadedAudio, ExtractedMedia,
    VideoMetadata,
};
pub use progress::{ProgressEvent, ProgressHandler};
pub use streamer::AudioStreamer;

use bytes::Bytes;
use futures_util::Stream;

pub async fn download_audio<P: AsRef<std::path::Path>>(
    url_or_id: &str,
    output_dir: P,
) -> Result<DownloadedAudio> {
    YoutubeAudioDownloader::new()
        .output_dir(output_dir)
        .format(AudioFormat::Mp3)
        .download(url_or_id)
        .await
}

pub async fn get_stream_info(url_or_id: &str) -> Result<AudioStreamResponse> {
    AudioStreamer::default().get_stream_response(url_or_id).await
}

pub async fn stream_audio(
    url_or_id: &str,
) -> Result<(VideoMetadata, impl Stream<Item = reqwest::Result<Bytes>>)> {
    AudioStreamer::default().stream_bytes(url_or_id).await
}

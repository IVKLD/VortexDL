use yt_audio_downloader::{AudioFormat, ProgressEvent, YoutubeAudioDownloader};

use crate::{
    downloader::DownloadTask,
    providers::{ProgressCallback, ProviderError, youtube::YouTubeProvider},
};

impl YouTubeProvider {
    pub async fn download(
        &self,
        task: &DownloadTask,
        url_or_id: &str,
        on_progress: Option<ProgressCallback>,
    ) -> Result<(), ProviderError> {
        YoutubeAudioDownloader::new()
            .output_file(&task.file_path)
            .format(AudioFormat::Mp3)
            .on_progress(move |event| {
                if let ProgressEvent::Downloading {
                    bytes_downloaded,
                    total_bytes,
                    ..
                } = event
                    && let Some(ref cb) = on_progress
                {
                    cb(bytes_downloaded, total_bytes);
                }
            })
            .download(url_or_id)
            .await?;

        Ok(())
    }
}

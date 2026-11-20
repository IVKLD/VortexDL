use yt_audio_downloader::AudioFormat;

use crate::{
    downloader::DownloadTask,
    infra,
    providers::{ProgressCallback, ProviderError, youtube::YouTubeProvider},
};

impl YouTubeProvider {
    pub async fn download(
        &self,
        task: &DownloadTask,
        url_or_id: &str,
        on_progress: Option<ProgressCallback>,
    ) -> Result<(), ProviderError> {
        self.client
            .download(
                url_or_id,
                &task.file_path,
                AudioFormat::Mp3,
                on_progress.as_ref(),
            )
            .await?;

        infra::verify_audio_file(&task.file_path, 0).await?;
        Ok(())
    }
}

use url::Url;

use crate::{
    downloader::DownloadTask,
    infra,
    providers::{ProgressCallback, ProviderError, soundcloud::SoundCloudProvider},
};

impl SoundCloudProvider {
    pub async fn download(
        &self,
        task: &mut DownloadTask,
        on_progress: Option<ProgressCallback>,
    ) -> Result<(), ProviderError> {
        let stream = self.resolve_stream(None, task.track.id).await?;
        if task.track.permalink_url.is_none() {
            task.track.permalink_url = Url::parse(&stream.url).ok();
        }

        self.client
            .download_stream(&stream, &task.file_path, on_progress.as_ref())
            .await?;

        infra::verify_audio_file(&task.file_path, 0).await?;
        Ok(())
    }
}

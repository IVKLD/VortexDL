use std::path::PathBuf;

use tokio::sync::watch;
use url::Url;

use crate::{
    domain::{AudioFormat, DiscoveredTrack, SourcePlatform},
    downloader::{cancellation, error::DownloadError, types::DownloadTask},
    infra::FileService,
    providers::{ProgressCallback, ProviderError, SoundCloudProvider, YouTubeProvider},
};

pub struct DownloadedTrackInfo {
    pub file_path: PathBuf,
    pub size: u64,
    pub format: AudioFormat,
}

#[derive(Clone)]
pub struct TrackDownloader {
    pub soundcloud: SoundCloudProvider,
    pub youtube: YouTubeProvider,
}

impl TrackDownloader {
    pub fn new(soundcloud: SoundCloudProvider, youtube: YouTubeProvider) -> Self {
        Self {
            soundcloud,
            youtube,
        }
    }

    pub async fn download(
        &self,
        task: &mut DownloadTask,
        cancel_rx: Option<watch::Receiver<bool>>,
        on_progress: Option<ProgressCallback>,
    ) -> Result<Option<DownloadedTrackInfo>, DownloadError> {
        let download_future = self.execute_download(task, on_progress);
        let result = cancellation::run_with_cancellation(cancel_rx, download_future).await;

        match result {
            Some(Ok(())) => {
                let size = FileService::file_size(&task.file_path)
                    .await
                    .unwrap_or(0);
                let format = AudioFormat::from_path(&task.file_path);
                Ok(Some(DownloadedTrackInfo {
                    file_path: task.file_path.clone(),
                    size,
                    format,
                }))
            }
            Some(Err(err)) => Err(err),
            None => {
                let _ = FileService::remove_file_if_exists(&task.file_path).await;
                Ok(None)
            }
        }
    }

    async fn execute_download(
        &self,
        task: &mut DownloadTask,
        on_progress: Option<ProgressCallback>,
    ) -> Result<(), DownloadError> {
        let platform = SourcePlatform::detect_from_optional_url(
            task.track.permalink_url.as_ref().map(|u| u.as_str()),
        );

        match platform {
            SourcePlatform::YouTube => {
                let url_or_id = task
                    .track
                    .permalink_url
                    .as_ref()
                    .map(|u| u.as_str())
                    .unwrap_or("");
                self.youtube.download(task, url_or_id, on_progress).await?;
                Ok(())
            }
            SourcePlatform::SoundCloud => {
                self.soundcloud.download(task, on_progress).await?;
                Ok(())
            }
            SourcePlatform::Local => Err(DownloadError::LocalTrack {
                name: task.display_name(),
                id: task.track.id,
            }),
        }
    }

    pub async fn discover_tracks(&self, url: &Url) -> Result<Vec<DiscoveredTrack>, DownloadError> {
        let platform = SourcePlatform::detect_from_url_str(url.as_str());
        let tracks = match platform {
            SourcePlatform::YouTube => self.youtube.discover(url).await?,
            SourcePlatform::SoundCloud => self.soundcloud.discover(url).await?,
            SourcePlatform::Local => {
                return Err(ProviderError::UnsupportedPlatform(url.to_string()).into());
            }
        };
        Ok(tracks)
    }

    pub fn spawn_artwork_fetch(
        &self,
        task: &DownloadTask,
    ) -> Option<tokio::task::JoinHandle<Option<Vec<u8>>>> {
        let platform = SourcePlatform::detect_from_optional_url(
            task.track.permalink_url.as_ref().map(|u| u.as_str()),
        );
        match platform {
            SourcePlatform::SoundCloud => self
                .soundcloud
                .spawn_artwork_fetch(task.track.artwork_url.as_ref()),
            SourcePlatform::YouTube | SourcePlatform::Local => None,
        }
    }
}

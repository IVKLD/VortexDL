use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::SystemTime,
};

use futures::stream::{self, StreamExt};
use soundcloud_rs::Client as SoundcloudClient;
use url::Url;

use crate::{
    api::{
        errors::{ApiError, ErrorCode},
        types::{ApiStatus, DownloadStartResponse},
    },
    devices::DeviceService,
    domain::DiscoveredTrack,
    downloader::{
        error::DownloadError,
        manager::DownloadManager,
        types::{DownloadItem, DownloadTask, ServerEvent},
        worker::{DownloadedTrackInfo, TrackDownloader},
    },
    infra::{EventBus, net::HttpClientPool, system_time_to_secs},
    library::{LibraryService, RegisterDownloadedTrackArgs},
    providers::{ProgressCallback, SoundCloudProvider, YouTubeProvider},
    settings::SettingsManager,
};

#[derive(Clone)]
pub struct DownloadService {
    library: LibraryService,
    manager: Arc<DownloadManager>,
    worker: TrackDownloader,
    settings: SettingsManager,
    events: EventBus,
}

impl DownloadService {
    pub fn new(
        library: LibraryService,
        soundcloud_client: Arc<SoundcloudClient>,
        manager: Arc<DownloadManager>,
        settings: SettingsManager,
        http_clients: HttpClientPool,
        events: EventBus,
    ) -> Self {
        let soundcloud =
            SoundCloudProvider::new(soundcloud_client, settings.clone(), http_clients.clone());
        let youtube = YouTubeProvider::new(settings.clone(), http_clients);
        let worker = TrackDownloader::new(soundcloud, youtube);

        Self {
            library,
            manager,
            worker,
            settings,
            events,
        }
    }

    pub fn manager(&self) -> &Arc<DownloadManager> {
        &self.manager
    }

    pub fn worker(&self) -> &TrackDownloader {
        &self.worker
    }

    pub fn events(&self) -> &EventBus {
        &self.events
    }

    pub fn get_queue(&self) -> Vec<DownloadItem> {
        self.manager.get_queue()
    }

    pub fn get_reserved_urls(&self) -> Vec<String> {
        self.manager.get_reserved_urls()
    }

    pub fn remove_from_queue(&self, id: i64) {
        self.manager.remove_task(id);
    }

    pub fn is_idle(&self) -> bool {
        self.manager.is_idle()
    }

    pub async fn enqueue(&self, url: Url) -> Result<DownloadStartResponse, ApiError> {
        if !self.manager.reserve_url(&url) {
            return Err(ApiError::conflict("This URL is already being processed")
                .with_code(ErrorCode::AlreadyProcessing));
        }

        self.events.publish(ServerEvent::SyncStarted {
            url: url.to_string(),
        });

        let service = self.clone();
        let target_url = url.clone();
        tokio::spawn(async move {
            struct Guard<'a>(&'a DownloadManager, &'a EventBus, &'a Url);
            impl Drop for Guard<'_> {
                fn drop(&mut self) {
                    self.0.release_url(self.2);
                    self.1.publish(ServerEvent::SyncFinished {
                        url: Some(self.2.to_string()),
                    });
                }
            }

            let _guard = Guard(&service.manager, &service.events, &target_url);
            if let Err(e) = service.download_url(&target_url).await {
                service.events.publish(ServerEvent::Error {
                    message: format!("Download failed: {e}"),
                });
            }
        });

        Ok(DownloadStartResponse {
            status: ApiStatus::Queued,
            message: format!("Started for: {url}"),
        })
    }

    pub async fn download_url(&self, url: &Url) -> Result<(), DownloadError> {
        let all_tracks = self.worker.discover_tracks(url).await?;
        let to_download = self.filter_pending_tracks(all_tracks).await;

        if !to_download.is_empty() {
            for track in &to_download {
                self.manager.add_task(track.clone());
            }
            self.download_batch(to_download).await;
        } else {
            tracing::info!("Everything synced!");
        }

        self.finalize_sync().await;
        Ok(())
    }

    pub async fn discover_tracks(&self, url: &Url) -> Result<Vec<DiscoveredTrack>, DownloadError> {
        self.worker.discover_tracks(url).await
    }

    async fn filter_pending_tracks(&self, tracks: Vec<DiscoveredTrack>) -> Vec<DiscoveredTrack> {
        let mut pending = Vec::with_capacity(tracks.len());
        for track in tracks {
            let exists = self.library.is_track_downloaded(track.id).await;
            if exists {
                tracing::info!(
                    "Skipping {} - {} (already downloaded)",
                    track.artist,
                    track.title
                );
            } else {
                pending.push(track);
            }
        }
        pending
    }

    async fn download_batch(&self, tracks: Vec<DiscoveredTrack>) {
        let total_tracks = tracks.len();

        let (max_concurrent, naming_template, output_dir) = {
            let s = self.settings.read().await;
            (
                s.downloads.max_concurrent as usize,
                s.downloads.naming_template.clone(),
                PathBuf::from(&s.downloads.output_path),
            )
        };

        let results: Vec<bool> = stream::iter(tracks)
            .map(|track| {
                let service = self.clone();
                let naming_template = naming_template.clone();
                let output_dir = output_dir.clone();

                async move {
                    service
                        .process_track(track, &naming_template, &output_dir)
                        .await
                }
            })
            .buffer_unordered(max_concurrent)
            .collect()
            .await;

        let succeeded = results.iter().filter(|&&ok| ok).count();
        let failed = total_tracks.saturating_sub(succeeded);
        tracing::info!("Sync complete. {succeeded} downloaded, {failed} failed or canceled.");
    }

    async fn process_track(
        &self,
        track: DiscoveredTrack,
        naming_template: &str,
        output_dir: &Path,
    ) -> bool {
        let mut task = DownloadTask::new(&track, naming_template, output_dir);
        let cancel_rx = self.manager.get_cancel_receiver(task.track.id);

        self.manager.update_downloading(task.track.id);

        let manager = self.manager.clone();
        let track_id = task.track.id;
        let on_progress: ProgressCallback = Arc::new(move |pos, total| {
            manager.update_progress(track_id, pos, total);
        });

        let artwork_handle = self.worker.spawn_artwork_fetch(&task);
        let download_result = self
            .worker
            .download(&mut task, cancel_rx, Some(on_progress))
            .await;

        match download_result {
            Ok(Some(info)) => {
                self.finalize_track(task, info, artwork_handle).await;
                true
            }
            Ok(None) => {
                tracing::info!("Track download canceled: {}", task.display_name());
                if let Some(h) = artwork_handle {
                    h.abort();
                }
                false
            }
            Err(err) => {
                self.record_failure(&task, &err);
                false
            }
        }
    }

    async fn finalize_track(
        &self,
        task: DownloadTask,
        info: DownloadedTrackInfo,
        artwork_handle: Option<tokio::task::JoinHandle<Option<Vec<u8>>>>,
    ) {
        tracing::info!("[OK] {}", task.display_name());

        let artwork_data = match artwork_handle {
            Some(handle) => handle.await.unwrap_or_default(),
            None => None,
        };

        let permalink_url = task.track.permalink_url.clone();
        let track_id = task.track.id;
        let size = info.size;

        if let Err(err) = self
            .library
            .register_downloaded_track(RegisterDownloadedTrackArgs {
                id: track_id,
                path: task.file_path.clone(),
                title: task.track.title.clone(),
                artist: task.track.artist.clone(),
                artwork_url: task.track.artwork_url.clone(),
                source_url: permalink_url.clone(),
                artwork_data,
                size,
            })
            .await
        {
            tracing::warn!(track_id, "Failed to register downloaded track: {err:#}");
        }

        let now = system_time_to_secs(SystemTime::now());
        self.manager.update_finished(
            task.track.id,
            info.format,
            now,
            permalink_url.as_ref(),
            size,
        );
    }

    fn record_failure(&self, task: &DownloadTask, err: &DownloadError) {
        let short_err = err.user_message();
        self.manager.update_failed(task.track.id, &short_err);
        tracing::error!("[ERROR] {} — {short_err}", task.display_name());
    }

    async fn finalize_sync(&self) {
        let device_service = DeviceService::new(self.library.clone(), self.settings.clone());
        device_service.sync_connected().await;
        self.worker.soundcloud.sync_client_id().await;
    }
}

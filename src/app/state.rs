use std::sync::Arc;

use soundcloud_rs::Client as SoundcloudClient;

use crate::{
    devices::DeviceService,
    downloader::{DownloadManager, DownloadService},
    infra::{EventBus, net::HttpClientPool},
    library::LibraryService,
    providers::{SoundCloudProvider, YouTubeProvider},
    search::{SearchCache, SearchService},
    settings::SettingsManager,
    stream::{StreamCache, StreamService},
};

#[derive(Clone)]
pub struct AppState {
    pub soundcloud: Arc<SoundcloudClient>,
    pub library: LibraryService,
    pub download_manager: Arc<DownloadManager>,
    pub events: EventBus,
    pub settings: SettingsManager,
    pub search_cache: SearchCache,
    pub stream_cache: StreamCache,
    pub http_clients: HttpClientPool,
}

impl AppState {
    pub fn new(
        soundcloud: SoundcloudClient,
        library: LibraryService,
        settings: SettingsManager,
    ) -> Self {
        let events = EventBus::default();
        let download_manager = Arc::new(DownloadManager::new(events.clone()));

        Self {
            soundcloud: Arc::new(soundcloud),
            library,
            download_manager,
            events,
            settings,
            search_cache: SearchCache::default(),
            stream_cache: StreamCache::default(),
            http_clients: HttpClientPool::default(),
        }
    }

    pub fn download_service(&self) -> DownloadService {
        DownloadService::new(
            self.library.clone(),
            self.soundcloud.clone(),
            self.download_manager.clone(),
            self.settings.clone(),
            self.http_clients.clone(),
            self.events.clone(),
        )
    }

    pub fn device_service(&self) -> DeviceService {
        DeviceService::new(self.library.clone(), self.settings.clone())
    }

    pub fn library_service(&self) -> LibraryService {
        self.library.clone()
    }

    pub fn search_service(&self) -> SearchService {
        SearchService::new(
            self.soundcloud_provider(),
            self.youtube_provider(),
            self.search_cache.clone(),
        )
    }

    pub fn stream_service(&self) -> StreamService {
        StreamService::new(
            self.library.clone(),
            self.stream_cache.clone(),
            self.search_cache.clone(),
            self.http_clients.clone(),
            self.soundcloud_provider(),
            self.youtube_provider(),
        )
    }

    pub fn soundcloud_provider(&self) -> SoundCloudProvider {
        SoundCloudProvider::new(
            self.soundcloud.clone(),
            self.settings.clone(),
            self.http_clients.clone(),
        )
    }

    pub fn youtube_provider(&self) -> YouTubeProvider {
        YouTubeProvider::new(self.settings.clone(), self.http_clients.clone())
    }

    pub fn events(&self) -> &EventBus {
        &self.events
    }
}

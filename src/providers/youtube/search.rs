use yt_audio_downloader::{VideoMetadata, YoutubeAudioDownloader};

use crate::providers::youtube::YouTubeProvider;

impl YouTubeProvider {
    pub async fn client(&self) -> reqwest::Client {
        let proxy = self
            .settings
            .read()
            .await
            .network
            .get_proxy_url()
            .map(String::from);
        self.http_clients.get_or_create(proxy)
    }

    pub fn is_youtube_url(&self, query: &str) -> bool {
        yt_audio_downloader::is_youtube_url(query)
    }

    pub async fn fetch_metadata(&self, url: &str) -> Option<VideoMetadata> {
        let client = self.client().await;
        YoutubeAudioDownloader::new()
            .client(client)
            .fetch_metadata(url)
            .await
            .ok()
    }

    pub async fn search_page(
        &self,
        query: &str,
        continuation: Option<&str>,
    ) -> Option<(Vec<VideoMetadata>, Option<String>)> {
        let client = self.client().await;
        yt_audio_downloader::search_youtube_page_with_client(query, continuation, client)
            .await
            .ok()
    }
}

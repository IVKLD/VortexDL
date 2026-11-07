use yt_audio_downloader::VideoMetadata;

use crate::providers::youtube::YouTubeProvider;

impl YouTubeProvider {
    pub fn is_youtube_url(&self, query: &str) -> bool {
        yt_audio_downloader::is_youtube_url(query)
    }

    pub async fn fetch_metadata(&self, url: &str) -> Option<VideoMetadata> {
        self.client.fetch_metadata(url).await.ok()
    }

    pub async fn search_page(
        &self,
        query: &str,
        continuation: Option<&str>,
    ) -> Option<(Vec<VideoMetadata>, Option<String>)> {
        self.client.search_page(query, continuation).await.ok()
    }
}

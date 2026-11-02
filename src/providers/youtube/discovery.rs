use url::Url;
use yt_audio_downloader::YoutubeAudioDownloader;

use crate::{
    domain::DiscoveredTrack,
    providers::{ProviderError, youtube::YouTubeProvider},
};

impl YouTubeProvider {
    pub async fn discover(&self, url: &Url) -> Result<Vec<DiscoveredTrack>, ProviderError> {
        let url_str = url.as_str();

        if let Some(playlist_id) = yt_audio_downloader::extractor::extract_playlist_id(url_str) {
            let metadata_list = yt_audio_downloader::fetch_playlist(&playlist_id).await?;
            Ok(metadata_list.iter().map(DiscoveredTrack::from).collect())
        } else {
            let meta = YoutubeAudioDownloader::new()
                .fetch_metadata(url_str)
                .await?;
            Ok(vec![DiscoveredTrack::from(&meta)])
        }
    }
}

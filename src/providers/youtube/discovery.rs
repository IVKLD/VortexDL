use anyhow::{Result, anyhow};
use url::Url;
use yt_audio_downloader::YoutubeAudioDownloader;

use crate::domain::DiscoveredTrack;

pub async fn discover_youtube_tracks(url: &Url) -> Result<Vec<DiscoveredTrack>> {
    let url_str = url.as_str();

    if let Some(playlist_id) = yt_audio_downloader::extractor::extract_playlist_id(url_str) {
        let metadata_list = yt_audio_downloader::fetch_playlist(&playlist_id)
            .await
            .map_err(|e| anyhow!("Failed to fetch YouTube playlist: {e}"))?;

        Ok(metadata_list.iter().map(DiscoveredTrack::from).collect())
    } else {
        let meta = YoutubeAudioDownloader::new()
            .fetch_metadata(url_str)
            .await
            .map_err(|e| anyhow!("Failed to fetch YouTube track metadata: {e}"))?;

        Ok(vec![DiscoveredTrack::from(&meta)])
    }
}

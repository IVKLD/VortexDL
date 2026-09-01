use anyhow::{Result, anyhow};
use url::Url;
use yt_audio_downloader::{YoutubeAudioDownloader, extractor::extract_playlist_id};

use crate::{downloader::DiscoveredMusicTrack, utils::filename::parse_track_metadata};

fn youtube_meta_to_discovered(
    meta: &yt_audio_downloader::models::VideoMetadata,
) -> DiscoveredMusicTrack {
    let id = yt_audio_downloader::youtube_id_to_i64(&meta.id);
    let permalink_url = Url::parse(&format!("https://www.youtube.com/watch?v={}", meta.id)).ok();
    let artwork_url = meta
        .thumbnail_url
        .as_deref()
        .and_then(|u| Url::parse(u).ok());
    let (artist, title) = parse_track_metadata(&meta.title, &meta.author);
    DiscoveredMusicTrack {
        id,
        title,
        artist,
        artwork_url,
        permalink_url,
        duration_ms: Some(meta.duration_seconds * 1000),
    }
}

pub async fn discover_youtube_tracks(url: &Url) -> Result<Vec<DiscoveredMusicTrack>> {
    let url_str = url.as_str();

    if let Some(playlist_id) = extract_playlist_id(url_str) {
        let metadata_list = yt_audio_downloader::fetch_playlist(&playlist_id)
            .await
            .map_err(|e| anyhow!("Failed to fetch YouTube playlist: {e}"))?;

        Ok(metadata_list
            .iter()
            .map(youtube_meta_to_discovered)
            .collect())
    } else {
        let meta = YoutubeAudioDownloader::new()
            .fetch_metadata(url_str)
            .await
            .map_err(|e| anyhow!("Failed to fetch YouTube track metadata: {e}"))?;

        Ok(vec![youtube_meta_to_discovered(&meta)])
    }
}

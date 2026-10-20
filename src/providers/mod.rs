pub mod cache;
pub mod soundcloud;
pub mod youtube;

use anyhow::{Result, bail};
use tokio::task::JoinHandle;
use url::Url;

use crate::{
    domain::{DiscoveredTrack, SourcePlatform},
    downloader::{DownloadContext, DownloadTask},
};

#[derive(Debug, Clone)]
pub struct ResolvedStreamInfo {
    pub stream_url: String,
    pub proxy_used: Option<String>,
}

pub async fn discover_tracks(ctx: &DownloadContext, url: &Url) -> Result<Vec<DiscoveredTrack>> {
    let platform = SourcePlatform::detect_from_url_str(url.as_str());
    match platform {
        SourcePlatform::YouTube => youtube::discover_youtube_tracks(url).await,
        SourcePlatform::SoundCloud => soundcloud::resolve_tracks_from_url(ctx, url).await,
        SourcePlatform::Local => {
            bail!("Unsupported music URL: '{url}'. Supported platforms: SoundCloud and YouTube.");
        }
    }
}

pub async fn download_track(ctx: &DownloadContext, task: &mut DownloadTask) -> Result<()> {
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
            youtube::download_youtube_track(ctx, task, url_or_id).await
        }
        SourcePlatform::SoundCloud => {
            let stream = soundcloud::resolve_stream_source(ctx, task.track.id).await?;
            if task.track.permalink_url.is_none() {
                task.track.permalink_url = Url::parse(stream.url()).ok();
            }
            soundcloud::download_soundcloud_track(ctx, task, stream).await
        }
        SourcePlatform::Local => {
            bail!(
                "Cannot download local track '{}' (id: {}): track is already local or has no remote provider",
                task.display_name(),
                task.track.id
            );
        }
    }
}

pub fn spawn_artwork_fetch(
    ctx: &DownloadContext,
    task: &DownloadTask,
) -> Option<JoinHandle<Option<Vec<u8>>>> {
    let platform = SourcePlatform::detect_from_optional_url(
        task.track.permalink_url.as_ref().map(|u| u.as_str()),
    );
    match platform {
        SourcePlatform::SoundCloud => {
            soundcloud::spawn_artwork_fetch(ctx, task.track.artwork_url.as_ref())
        }
        SourcePlatform::YouTube | SourcePlatform::Local => None,
    }
}

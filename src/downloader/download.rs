use anyhow::Result;
use url::Url;

use super::{
    providers::{soundcloud, youtube},
    task::{Context, DownloadTask},
};
use crate::types::SourcePlatform;

pub fn spawn_artwork_fetch(
    ctx: &Context,
    task: &DownloadTask,
) -> Option<tokio::task::JoinHandle<Option<Vec<u8>>>> {
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

/// What: Downloads a single audio track using the appropriate platform provider (SoundCloud/YouTube).
/// Flow: Executed inside each parallel task spawned by `process_parallel_track`.
/// Why:  Detects the source platform from track metadata, resolves streams/URLs dynamically,
///       and delegates download execution to the dedicated platform module.
pub async fn download_track(ctx: &Context, task: &mut DownloadTask) -> Result<()> {
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
            anyhow::bail!(
                "Cannot download local track '{}' (id: {}): track is already local or has no remote provider",
                task.display_name(),
                task.track.id
            );
        }
    }
}

use anyhow::Result;
use url::Url;

use super::{complete, parallel::run_parallel_downloads, providers, task::Context};
use crate::types::DiscoveredMusicTrack;

/// What: End-to-end pipeline that takes a URL (SoundCloud/YouTube) and processes all tracks to completion.
/// Flow: Top-level entry point called by background download tasks from the API queue and CLI `download` command.
/// Why:  Coordinates 5 key stages: (1) platform detection, (2) track discovery, (3) filtering out already-downloaded
///       tracks, (4) concurrent downloads with rate-limiting, and (5) playlist ID sync and ADB sync.
pub async fn run_download_pipeline(ctx: &Context, url: &Url) -> Result<()> {
    let all_tracks = providers::discover_tracks(ctx, url).await?;

    let to_download = exclude_already_downloaded_tracks(ctx, all_tracks).await;

    if !to_download.is_empty() {
        if let Some(ref m) = ctx.dm {
            for track in &to_download {
                m.add_task(track.clone());
            }
        }
        run_parallel_downloads(ctx, to_download).await;
    } else {
        tracing::info!("Everything synced!");
    }

    complete::finalize_pipeline_sync(ctx, url).await?;

    Ok(())
}

async fn exclude_already_downloaded_tracks(
    ctx: &Context,
    tracks: Vec<DiscoveredMusicTrack>,
) -> Vec<DiscoveredMusicTrack> {
    let storage = ctx.storage.read().await;

    tracks
        .into_iter()
        .filter(|track| {
            let exists = storage.is_track_downloaded(track.id);
            if exists {
                tracing::info!(
                    "Skipping {} - {} (already downloaded)",
                    track.artist,
                    track.title
                );
            }
            !exists
        })
        .collect()
}

use std::collections::HashSet;

use anyhow::{Result, anyhow};
use colored::Colorize;

use crate::adb_device;

pub mod core;
pub(crate) mod discovery;
pub mod types;

use discovery::{
    DiscoveryContext, fetch_likes, fetch_playlist, fetch_track, resolve_with_feedback,
};
pub use types::{Context, TrackDownload};

/// Downloads and synchronizes a resource from a given URL.
pub async fn download(ctx: &Context, url: &str) -> Result<()> {
    let discovery_ctx = DiscoveryContext {
        client: &ctx.client,
        settings: &ctx.settings,
        dm: ctx.dm.as_ref(),
    };

    let resolve_res = resolve_with_feedback(&discovery_ctx, url, "Resolving URL...").await?;

    let all_tracks = match resolve_res.kind.as_str() {
        "user" | "likes" => fetch_likes(&discovery_ctx, resolve_res.id).await?,
        "playlist" => fetch_playlist(&discovery_ctx, resolve_res.id).await?,
        "track" => vec![fetch_track(&discovery_ctx, resolve_res.id).await?],
        _ => return Err(anyhow!("Unsupported resource kind: {}", resolve_res.kind)),
    };

    let mut to_download = Vec::with_capacity(all_tracks.len());
    let mut remote_ids = HashSet::with_capacity(all_tracks.len());
    let mut skipped = 0;

    {
        let storage_read = ctx.storage.read().await;
        for track in all_tracks {
            let track_download = TrackDownload {
                id: track.id,
                title: track.title.clone(),
                artist: track.artist.clone(),
                artwork_url: track.artwork_url.clone(),
                position: track.position,
            };

            remote_ids.insert(track.id);

            let already_exists = if let Some(data) = storage_read.tracks.get(&track.id) {
                data.path.exists()
            } else {
                false
            };

            if already_exists {
                println!(
                    "{} Skipping: {} - {}",
                    "[SKIP]".yellow().bold(),
                    track.artist,
                    track.title
                );
                skipped += 1;
            } else {
                to_download.push(track_download);
            }
        }
    }

    if skipped > 0 {
        println!("{} Skipped {} tracks.", "[INFO]".blue().bold(), skipped);
    }

    if let Some(ref m) = ctx.dm {
        for track in &to_download {
            m.add_task(
                track.id,
                track.title.clone(),
                track.artist.clone(),
                track.artwork_url.clone(),
            );
        }
    }

    if !to_download.is_empty() {
        core::run_download_batch(ctx, to_download).await;
    } else {
        println!("{} Everything synced!", "[INFO]".blue().bold());
    }

    let sync_mode = ctx.settings.read().await.downloads.sync_mode.clone();
    ctx.storage
        .write()
        .await
        .sync_storage(url, &remote_ids, &sync_mode)
        .await?;

    adb_device::sync_all_connected(ctx.storage.clone(), ctx.settings.clone()).await;

    Ok(())
}

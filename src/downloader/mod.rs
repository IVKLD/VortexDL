use std::collections::HashSet;

use anyhow::{Result, anyhow};
use colored::Colorize;

use crate::{adb_device, utils::metadata::update_track_position};

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

    let remote_ids: HashSet<i64> = all_tracks.iter().map(|track| track.id).collect();
    let to_download: Vec<TrackDownload> = {
        let mut storage_write = ctx.storage.write().await;
        all_tracks
            .into_iter()
            .filter_map(|track| process_track_download(&mut storage_write, track))
            .collect()
    };

    let skipped = remote_ids.len() - to_download.len();

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
                track.position,
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

fn process_track_download(
    storage: &mut crate::storage::MusicStorage,
    track: TrackDownload,
) -> Option<TrackDownload> {
    if let Some(data) = storage
        .tracks
        .get_mut(&track.id)
        .filter(|d| d.path.exists())
    {
        if data.position != track.position {
            data.position = track.position;
            let path = data.path.clone();
            let position = track.position;
            tokio::task::spawn_blocking(move || {
                let _ = update_track_position(&path.to_string_lossy(), position);
            });
        }
        println!(
            "{} {} - {}",
            "[SKIP]".yellow().bold(),
            track.artist,
            track.title
        );
        None
    } else {
        Some(track)
    }
}

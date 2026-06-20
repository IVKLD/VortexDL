use std::collections::HashSet;

use anyhow::{Result, anyhow};
use colored::Colorize;

use crate::{
    adb_device,
    api::download_manager,
    storage::MusicStorage,
    utils::{
        metadata::update_track_position,
        proxy::race_proxies,
        soundcloud::{init_client_with_settings, resolve_url, update_cached_client_id},
    },
};

pub mod core;
pub mod discovery;

use discovery::{
    discover_liked_tracks, discover_playlist_tracks, discover_single_track, init_progress_spinner,
};

pub use crate::types::core::{Context, DiscoveredTrack};

async fn discover_tracks_from_url(
    ctx: &Context,
    url: &str,
    client: &soundcloud_rs::Client,
) -> Result<Vec<DiscoveredTrack>> {
    let pb = init_progress_spinner(ctx, "Resolving URL...");
    let resolve_res = resolve_url(client, url).await;
    pb.finish_and_clear();
    let resolve_res = resolve_res?;

    let all_tracks = match resolve_res.kind.as_str() {
        "user" | "likes" => discover_liked_tracks(ctx, client, resolve_res.id).await?,
        "playlist" => discover_playlist_tracks(ctx, client, resolve_res.id).await?,
        "track" => vec![discover_single_track(client, resolve_res.id).await?],
        _ => return Err(anyhow!("Unsupported resource kind: {}", resolve_res.kind)),
    };

    Ok(all_tracks)
}

pub async fn download(ctx: &Context, url: &str) -> Result<()> {
    let all_tracks = match discover_tracks_from_url(ctx, url, &ctx.client).await {
        Ok(tracks) => tracks,
        Err(e) => {
            let settings = ctx.settings.read().await.clone();
            tracing::debug!("Direct discovery failed: {e}. Trying fallback proxies...");

            let ctx = ctx.clone();
            let url = url.to_string();

            race_proxies(&settings, |s, proxy| {
                let ctx = ctx.clone();
                let url = url.clone();
                async move {
                    let proxied_client = init_client_with_settings(&s, Some(&proxy)).await?;
                    discover_tracks_from_url(&ctx, &url, &proxied_client).await
                }
            })
            .await
            .map_err(|proxy_err| anyhow!("Discovery failed: {e} (proxies: {proxy_err})"))?
        }
    };

    let remote_ids: HashSet<i64> = all_tracks.iter().map(|track| track.id).collect();
    let to_download: Vec<DiscoveredTrack> = {
        let mut storage_write = ctx.storage.write().await;
        all_tracks
            .into_iter()
            .filter_map(|track| exclude_existing_track(&mut storage_write, track))
            .collect()
    };

    let skipped = remote_ids.len() - to_download.len();

    if skipped > 0 {
        println!("{} Skipped {} tracks.", "[INFO]".blue().bold(), skipped);
    }

    if let Some(ref m) = ctx.dm {
        for track in &to_download {
            m.add_task(download_manager::AddTaskArgs {
                id: track.id,
                title: track.title.clone(),
                artist: track.artist.clone(),
                artwork_url: track.artwork_url.clone(),
                position: track.position,
            });
        }
    }

    if !to_download.is_empty() {
        core::run_download_batch(ctx, to_download).await;
    } else {
        println!("{} Everything synced!", "[INFO]".blue().bold());
    }

    let sync_mode = ctx.settings.read().await.downloads.sync_mode;
    ctx.storage
        .write()
        .await
        .sync_storage(url, &remote_ids, &sync_mode)
        .await?;

    adb_device::sync_connected(ctx.storage.clone(), ctx.settings.clone()).await;

    update_cached_client_id(&ctx.client, &ctx.settings).await;

    Ok(())
}

fn exclude_existing_track(
    storage: &mut MusicStorage,
    track: DiscoveredTrack,
) -> Option<DiscoveredTrack> {
    if let Some(data) = storage
        .tracks
        .get_mut(&track.id)
        .filter(|d| d.path.exists())
    {
        if data.position != track.position {
            data.position = track.position;
            let path = data.path.clone();
            let position = track.position;
            let handle = tokio::task::spawn_blocking(move || {
                update_track_position(path, position)
            });
            tokio::spawn(async move {
                if let Ok(Err(e)) = handle.await {
                    tracing::warn!("Failed to update track position: {e}");
                }
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

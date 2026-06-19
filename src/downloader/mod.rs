use std::collections::HashSet;

use anyhow::{Result, anyhow};
use colored::Colorize;

use crate::{
    adb_device,
    storage::MusicStorage,
    utils::{
        metadata::update_track_position,
        proxy::race_proxies,
        soundcloud::{init_client_with_settings, resolve_url, update_cached_client_id},
    },
};

pub mod core;
pub mod discovery;

use discovery::{fetch_likes, fetch_playlist, fetch_track, show_feedback};

pub use crate::types::core::{Context, TrackDownload};

async fn resolve_and_fetch_tracks(
    ctx: &Context,
    url: &str,
    client: &soundcloud_rs::Client,
) -> Result<Vec<TrackDownload>> {
    let pb = show_feedback(ctx, "Resolving URL...");
    let resolve_res = resolve_url(client, url).await;
    pb.finish_and_clear();
    let resolve_res = resolve_res?;

    let all_tracks = match resolve_res.kind.as_str() {
        "user" | "likes" => fetch_likes(ctx, client, resolve_res.id).await?,
        "playlist" => fetch_playlist(ctx, client, resolve_res.id).await?,
        "track" => vec![fetch_track(client, resolve_res.id).await?],
        _ => return Err(anyhow!("Unsupported resource kind: {}", resolve_res.kind)),
    };

    Ok(all_tracks)
}

pub async fn download(ctx: &Context, url: &str) -> Result<()> {
    let all_tracks = match resolve_and_fetch_tracks(ctx, url, &ctx.client).await {
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
                    resolve_and_fetch_tracks(&ctx, &url, &proxied_client).await
                }
            })
            .await
            .map_err(|proxy_err| anyhow!("Discovery failed: {e} (proxies: {proxy_err})"))?
        }
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

    let sync_mode = ctx.settings.read().await.downloads.sync_mode;
    ctx.storage
        .write()
        .await
        .sync_storage(url, &remote_ids, &sync_mode)
        .await?;

    adb_device::sync_all_connected(ctx.storage.clone(), ctx.settings.clone()).await;

    update_cached_client_id(&ctx.client, &ctx.settings).await;

    Ok(())
}

fn process_track_download(
    storage: &mut MusicStorage,
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
                let _ = update_track_position(path, position);
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

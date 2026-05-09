use std::collections::HashSet;

use anyhow::{Result, anyhow};
use colored::Colorize;

use crate::{
    downloader::{
        Context,
        core::run_parallel_download,
        discovery::{
            DiscoveryContext, fetch_likes, fetch_playlist, fetch_track, resolve_with_feedback,
        },
    },
    models::SyncMode,
};

pub async fn dispatch_download(
    url: &str,
    sync_mode: SyncMode,
    ctx: &Context,
) -> Result<HashSet<i64>> {
    let discovery_ctx = DiscoveryContext {
        client: &ctx.client,
        config: &ctx.config,
        dm: ctx.dm.as_ref(),
    };

    let resolve_res = resolve_with_feedback(&discovery_ctx, url, "Resolving URL...").await?;

    let all_tracks = match resolve_res.kind.as_str() {
        "user" | "likes" => fetch_likes(&discovery_ctx, url).await?,
        "playlist" => fetch_playlist(&discovery_ctx, url).await?,
        "track" => vec![fetch_track(&discovery_ctx, resolve_res.id).await?],
        _ => {
            return Err(anyhow!("Unsupported resource kind: {}", resolve_res.kind));
        }
    };

    let mut to_download = Vec::new();
    let mut remote_ids = HashSet::new();
    let mut skipped = 0;

    {
        let storage_read = ctx.storage.read().await;
        for track in all_tracks {
            remote_ids.insert(track.id);
            if storage_read.tracks.contains_key(&track.id) {
                println!("{} Skipping: {}", "[SKIP]".yellow().bold(), track.filename);
                skipped += 1;
            } else {
                to_download.push(track);
            }
        }
    }

    if skipped > 0 {
        println!("{} Skipped {} tracks.", "[INFO]".blue().bold(), skipped);
    }

    if let Some(ref m) = ctx.dm {
        for track in &to_download {
            m.add_task(track.id, track.filename.clone(), track.artwork_url.clone())
                .await;
        }
    }

    if !to_download.is_empty() {
        run_parallel_download(ctx, to_download).await;
    } else {
        println!("{} Everything synced!", "[INFO]".blue().bold());
    }

    ctx.storage
        .write()
        .await
        .sync_storage(url, &remote_ids, &sync_mode)
        .await?;

    Ok(remote_ids)
}

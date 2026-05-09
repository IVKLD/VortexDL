use anyhow::{Result, anyhow};
use soundcloud_rs::Identifier;

use crate::downloader::{
    core::TrackDownload,
    discovery::{DiscoveryContext, resolve_with_feedback},
};

pub async fn fetch_playlist(ctx: &DiscoveryContext<'_>, url: &str) -> Result<Vec<TrackDownload>> {
    let resolve_res = resolve_with_feedback(ctx, url, "Resolving playlist URL...").await?;

    let playlist_id = Identifier::Id(resolve_res.id);
    let playlist = ctx.client.get_playlist(&playlist_id).await?;

    let collection = playlist
        .tracks
        .ok_or_else(|| anyhow!("No tracks found in playlist"))?;

    let tracks = collection
        .into_iter()
        .filter_map(|track| {
            let id = track.id?;

            let author = track
                .user
                .as_ref()
                .and_then(|u| u.username.as_deref())
                .unwrap_or("Unknown");

            let title = track.title.as_deref().unwrap_or("Unknown");
            let filename = format!("{} - {}", author, title);

            Some(TrackDownload {
                id,
                filename,
                artwork_url: track.artwork_url,
            })
        })
        .collect();

    Ok(tracks)
}

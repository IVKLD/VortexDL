use anyhow::{Result, anyhow};
use soundcloud_rs::Identifier;

use crate::downloader::{
    DiscoveryContext, TrackDownload,
    discovery::{extract_artist, extract_title},
};

pub async fn fetch_playlist(ctx: &DiscoveryContext<'_>, id: i64) -> Result<Vec<TrackDownload>> {
    let playlist = ctx.client.get_playlist(&Identifier::Id(id)).await?;
    let collection = playlist
        .tracks
        .ok_or_else(|| anyhow!("No tracks found in playlist"))?;

    Ok(collection
        .into_iter()
        .enumerate()
        .filter_map(|(i, track)| {
            Some(TrackDownload {
                id: track.id?,
                title: extract_title(track.title.as_deref()),
                artist: extract_artist(track.user.as_ref()),
                artwork_url: track.artwork_url,
                position: Some(i as u32),
            })
        })
        .collect())
}

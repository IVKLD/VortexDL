use anyhow::{Result, anyhow};
use soundcloud_rs::Identifier;

use crate::downloader::{TrackDownload, discovery::DiscoveryContext};

pub async fn fetch_playlist(ctx: &DiscoveryContext<'_>, id: i64) -> Result<Vec<TrackDownload>> {
    let playlist_id = Identifier::Id(id);
    let playlist = ctx.client.get_playlist(&playlist_id).await?;

    let collection = playlist
        .tracks
        .ok_or_else(|| anyhow!("No tracks found in playlist"))?;

    let tracks = collection
        .into_iter()
        .enumerate()
        .filter_map(|(i, track)| {
            let id = track.id?;

            let artist = track
                .user
                .as_ref()
                .and_then(|u| u.username.as_deref())
                .map(|t| t.to_string())
                .unwrap_or("Unknown".to_string());

            let title = track
                .title
                .as_deref()
                .map(|t| t.to_string())
                .unwrap_or("Unknown".to_string());

            let artwork_url = track.artwork_url;

            Some(TrackDownload {
                id,
                title,
                artist,
                artwork_url,
                position: Some(i as u32),
            })
        })
        .collect();

    Ok(tracks)
}

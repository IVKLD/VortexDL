use anyhow::Result;
use soundcloud_rs::Identifier;

use crate::downloader::{core::TrackDownload, discovery::DiscoveryContext};

pub async fn fetch_track(ctx: &DiscoveryContext<'_>, id: i64) -> Result<TrackDownload> {
    let track = ctx.client.get_track(&Identifier::Id(id)).await?;

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

    Ok(TrackDownload {
        id,
        title,
        artist,
        artwork_url,
    })
}

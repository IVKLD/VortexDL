use anyhow::Result;
use soundcloud_rs::Identifier;

use crate::downloader::{core::TrackDownload, discovery::DiscoveryContext};

pub async fn fetch_track(ctx: &DiscoveryContext<'_>, id: i64) -> Result<TrackDownload> {
    let track = ctx.client.get_track(&Identifier::Id(id)).await?;

    let author = track
        .user
        .as_ref()
        .and_then(|u| u.username.as_deref())
        .unwrap_or("Unknown");

    let title = track.title.as_deref().unwrap_or("Unknown");
    let filename = format!("{} - {}", author, title);

    Ok(TrackDownload {
        id,
        filename,
        artwork_url: track.artwork_url,
    })
}

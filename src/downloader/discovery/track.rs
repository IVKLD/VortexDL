use anyhow::Result;
use soundcloud_rs::Identifier;

use crate::downloader::{
    DiscoveryContext, TrackDownload,
    discovery::{extract_artist, extract_title},
};

pub async fn fetch_track(ctx: &DiscoveryContext<'_>, id: i64) -> Result<TrackDownload> {
    let track = ctx.client.get_track(&Identifier::Id(id)).await?;

    Ok(TrackDownload {
        id,
        title: extract_title(track.title.as_deref()),
        artist: extract_artist(track.user.as_ref()),
        artwork_url: track.artwork_url,
        position: None,
    })
}

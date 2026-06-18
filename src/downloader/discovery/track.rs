use anyhow::Result;
use soundcloud_rs::{Client, Identifier};

use crate::downloader::TrackDownload;

pub async fn fetch_track(client: &Client, id: i64) -> Result<TrackDownload> {
    let track = client.get_track(&Identifier::Id(id)).await?;

    Ok(TrackDownload::new(
        id,
        track.title.as_deref(),
        track.user.as_ref(),
        track.artwork_url,
        None,
    ))
}

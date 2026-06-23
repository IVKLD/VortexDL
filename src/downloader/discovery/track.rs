use anyhow::Result;
use soundcloud_rs::{Client, Identifier};

use crate::types::core::DiscoveredMusicTrack;

pub async fn discover_single_track(client: &Client, id: i64) -> Result<DiscoveredMusicTrack> {
    let track = client.get_track(&Identifier::Id(id)).await?;
    DiscoveredMusicTrack::from_track(track).ok_or_else(|| anyhow::anyhow!("Track missing required ID"))
}

use anyhow::{Result, anyhow};
use soundcloud_rs::{Client, Identifier};

use crate::{
    downloader::{Context, discovery::init_progress_spinner},
    types::DiscoveredMusicTrack,
};

pub async fn discover_playlist_tracks(
    ctx: &Context,
    client: &Client,
    id: i64,
) -> Result<Vec<DiscoveredMusicTrack>> {
    let playlist = client.get_playlist(&Identifier::Id(id)).await?;
    let collection = playlist
        .tracks
        .ok_or_else(|| anyhow!("No tracks found in playlist"))?;

    let mut tracks: Vec<DiscoveredMusicTrack> = collection
        .into_iter()
        .filter_map(DiscoveredMusicTrack::from_track)
        .collect();

    let missing_ids: Vec<i64> = tracks
        .iter()
        .filter(|track| track.title == "Unknown")
        .map(|track| track.id)
        .collect();

    if !missing_ids.is_empty() {
        let pb = init_progress_spinner(ctx, "Resolving playlist track metadata...");
        for chunk in missing_ids.chunks(50) {
            if let Ok(fetched_tracks) = client.get_tracks(chunk).await {
                for track in fetched_tracks {
                    let Some(track_id) = track.id else {
                        continue;
                    };
                    if let (Some(local_track), Some(updated)) = (
                        tracks.iter_mut().find(|t| t.id == track_id),
                        DiscoveredMusicTrack::from_track(track),
                    ) {
                        *local_track = updated;
                    }
                }
            }
        }
        pb.finish_and_clear();
    }

    Ok(tracks)
}

use std::collections::HashMap;

use anyhow::{Result, anyhow};
use soundcloud_rs::{Client, Identifier};

use super::helpers::init_progress_spinner;
use crate::{downloader::Context, types::DiscoveredMusicTrack};

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

        let index: HashMap<i64, usize> =
            tracks.iter().enumerate().map(|(i, t)| (t.id, i)).collect();

        for chunk in missing_ids.chunks(50) {
            if let Ok(fetched_tracks) = client.get_tracks(chunk).await {
                for track in fetched_tracks {
                    let Some(track_id) = track.id else { continue };
                    let Some(updated) = DiscoveredMusicTrack::from_track(track) else {
                        continue;
                    };
                    if let Some(&idx) = index.get(&track_id) {
                        tracks[idx] = updated;
                    }
                }
            }
        }
        pb.finish_and_clear();
    }

    Ok(tracks)
}

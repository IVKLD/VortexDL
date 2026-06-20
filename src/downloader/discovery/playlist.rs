use std::collections::HashMap;

use anyhow::{Result, anyhow};
use soundcloud_rs::{Client, Identifier, Track};

use crate::downloader::{Context, DiscoveredTrack, discovery::init_progress_spinner};

pub async fn discover_playlist_tracks(ctx: &Context, client: &Client, id: i64) -> Result<Vec<DiscoveredTrack>> {
    let playlist = client.get_playlist(&Identifier::Id(id)).await?;
    let collection = playlist
        .tracks
        .ok_or_else(|| anyhow!("No tracks found in playlist"))?;

    let mut tracks: Vec<DiscoveredTrack> = collection
        .into_iter()
        .enumerate()
        .filter_map(|(i, track)| {
            DiscoveredTrack::from_track(track).map(|t| t.with_position(i.try_into().ok()))
        })
        .collect();

    let missing_ids: Vec<i64> = tracks
        .iter()
        .filter(|track| track.title == "Unknown")
        .map(|track| track.id)
        .collect();

    if !missing_ids.is_empty() {
        let pb = init_progress_spinner(ctx, "Resolving playlist track metadata...");
        for chunk in missing_ids.chunks(50) {
            let ids_str = chunk
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(",");

            let query = HashMap::from([("ids", ids_str)]);

            if let Ok(fetched_tracks) = client.get::<_, Vec<Track>>("tracks", Some(&query)).await {
                for track in fetched_tracks {
                    let Some(track_id) = track.id else {
                        continue;
                    };
                    if let (Some(local_track), Some(mut updated)) = (
                        tracks.iter_mut().find(|t| t.id == track_id),
                        DiscoveredTrack::from_track(track),
                    ) {
                        updated.position = local_track.position;
                        *local_track = updated;
                    }
                }
            }
        }
        pb.finish_and_clear();
    }

    Ok(tracks)
}

use std::collections::HashMap;

use anyhow::{Result, anyhow};
use soundcloud_rs::{Client, Identifier, Track};

use crate::downloader::{
    Context, TrackDownload,
    discovery::show_feedback,
};

pub async fn fetch_playlist(
    ctx: &Context,
    client: &Client,
    id: i64,
) -> Result<Vec<TrackDownload>> {
    let playlist = client.get_playlist(&Identifier::Id(id)).await?;
    let collection = playlist
        .tracks
        .ok_or_else(|| anyhow!("No tracks found in playlist"))?;

    let mut tracks: Vec<TrackDownload> = collection
        .into_iter()
        .enumerate()
        .filter_map(|(i, track)| {
            track.id.map(|id| {
                TrackDownload::new(
                    id,
                    track.title.as_deref(),
                    track.user.as_ref(),
                    track.artwork_url,
                    Some(i as u32),
                )
            })
        })
        .collect();

    let missing_ids: Vec<i64> = tracks
        .iter()
        .filter(|t| t.title == "Unknown")
        .map(|t| t.id)
        .collect();

    if !missing_ids.is_empty() {
        let pb = show_feedback(ctx, "Resolving playlist track metadata...");
        for chunk in missing_ids.chunks(50) {
            let ids_str = chunk
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(",");

            let query = HashMap::from([("ids", ids_str)]);

            if let Ok(fetched_tracks) = client.get::<_, Vec<Track>>("tracks", Some(&query)).await {
                for ft in fetched_tracks {
                    if let Some(t) = ft.id.and_then(|fid| tracks.iter_mut().find(|t| t.id == fid)) {
                        *t = TrackDownload::new(
                            t.id,
                            ft.title.as_deref(),
                            ft.user.as_ref(),
                            ft.artwork_url,
                            t.position,
                        );
                    }
                }
            }
        }
        pb.finish_and_clear();
    }

    Ok(tracks)
}

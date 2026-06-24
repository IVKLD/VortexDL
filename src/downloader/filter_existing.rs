use crate::{
    downloader::{Context, DiscoveredMusicTrack},
    utils::metadata::update_track_position,
};

pub async fn exclude_already_downloaded_tracks(
    ctx: &Context,
    tracks: Vec<DiscoveredMusicTrack>,
) -> Vec<DiscoveredMusicTrack> {
    let mut position_updates: Vec<(std::path::PathBuf, Option<u32>)> = Vec::new();

    let to_download: Vec<DiscoveredMusicTrack> = {
        let mut storage_write = ctx.storage.write().await;

        tracks
            .into_iter()
            .filter_map(|track| {
                if let Some(data) = storage_write
                    .tracks
                    .get_mut(&track.id)
                    .filter(|d| d.path.exists() && !d.is_archived())
                {
                    if data.position != track.position {
                        data.position = track.position;
                        position_updates.push((data.path.clone(), track.position));
                    }
                    tracing::info!(
                        "Skipping {} - {} (already downloaded)",
                        track.artist,
                        track.title
                    );
                    None
                } else {
                    Some(track)
                }
            })
            .collect()
    };

    for (path, position) in position_updates {
        tokio::task::spawn_blocking(move || {
            if let Err(e) = update_track_position(path, position) {
                tracing::warn!("Failed to update track position: {e}");
            }
        });
    }

    to_download
}

use crate::downloader::{Context, DiscoveredMusicTrack};

pub async fn exclude_already_downloaded_tracks(
    ctx: &Context,
    tracks: Vec<DiscoveredMusicTrack>,
) -> Vec<DiscoveredMusicTrack> {
    let storage_write = ctx.storage.write().await;

    tracks
        .into_iter()
        .filter(|track| {
            if storage_write
                .tracks
                .get(&track.id)
                .map_or(false, |d| d.path.exists() && !d.is_archived())
            {
                tracing::info!(
                    "Skipping {} - {} (already downloaded)",
                    track.artist,
                    track.title
                );
                false
            } else {
                true
            }
        })
        .collect()
}

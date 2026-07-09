use crate::downloader::{Context, DiscoveredMusicTrack};

pub async fn exclude_already_downloaded_tracks(
    ctx: &Context,
    tracks: Vec<DiscoveredMusicTrack>,
) -> Vec<DiscoveredMusicTrack> {
    let storage = ctx.storage.read().await;

    tracks
        .into_iter()
        .filter(|track| {
            if storage
                .tracks
                .get(&track.id)
                .is_some_and(|d| d.path.exists() && !d.is_archived())
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

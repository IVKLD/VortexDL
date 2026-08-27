use crate::downloader::{Context, DiscoveredMusicTrack};

pub async fn exclude_already_downloaded_tracks(
    ctx: &Context,
    tracks: Vec<DiscoveredMusicTrack>,
) -> Vec<DiscoveredMusicTrack> {
    let storage = ctx.storage.read().await;

    tracks
        .into_iter()
        .filter(|track| {
            let exists = storage
                .tracks
                .get(&track.id)
                .is_some_and(|d| d.path.exists() && !d.is_archived());
            if exists {
                tracing::info!(
                    "Skipping {} - {} (already downloaded)",
                    track.artist,
                    track.title
                );
            }
            !exists
        })
        .collect()
}

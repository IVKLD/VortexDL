pub mod soundcloud;
pub mod youtube;

use std::time::{Duration, Instant};

pub use soundcloud::resolve_soundcloud_stream;
pub use youtube::resolve_youtube_stream;

use crate::api::{errors::ApiError, state::AppState};

pub async fn resolve_stream_url(
    state: &AppState,
    id: i64,
    url_param: Option<String>,
) -> Result<String, ApiError> {
    if let Some((cached_url, instant)) = state.cache.streams.read().await.get(&id)
        && instant.elapsed() < Duration::from_secs(3 * 3600)
    {
        return Ok(cached_url.clone());
    }

    let target_url = if let Some(u) = url_param {
        Some(u)
    } else if let Some(vid) = state.cache.youtube_ids.read().await.get(&id) {
        Some(format!("https://www.youtube.com/watch?v={vid}"))
    } else {
        state
            .storage
            .read()
            .await
            .tracks
            .get(&id)
            .and_then(|t| t.metadata.source_url.clone())
    };

    let stream_url = if let Some(ref u) = target_url
        && yt_audio_downloader::is_youtube_url(u)
    {
        resolve_youtube_stream(state, u).await?
    } else {
        resolve_soundcloud_stream(state, id).await?
    };

    state
        .cache
        .streams
        .write()
        .await
        .insert(id, (stream_url.clone(), Instant::now()));

    Ok(stream_url)
}

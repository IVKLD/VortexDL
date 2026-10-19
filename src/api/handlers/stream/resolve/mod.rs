pub mod soundcloud;
pub mod youtube;

pub use soundcloud::resolve_soundcloud_stream;
pub use youtube::resolve_youtube_stream;

use crate::{
    api::{errors::ApiError, state::AppState},
    types::SourcePlatform,
};

pub async fn resolve_stream_url(
    state: &AppState,
    id: i64,
    url_param: Option<String>,
) -> Result<String, ApiError> {
    if let Some(cached_url) = state.cache.get_stream(id).await {
        return Ok(cached_url);
    }

    let target_url = if let Some(u) = url_param {
        Some(u)
    } else if let Some(vid) = state.cache.get_youtube_id(id).await {
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

    let stream_url = match target_url
        .as_deref()
        .map(SourcePlatform::detect_from_url_str)
    {
        Some(SourcePlatform::YouTube) => {
            let u = target_url.as_deref().unwrap();
            resolve_youtube_stream(state, u).await?
        }
        _ => resolve_soundcloud_stream(state, id).await?,
    };

    state.cache.insert_stream(id, stream_url.clone()).await;

    Ok(stream_url)
}

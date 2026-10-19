pub mod soundcloud;
pub mod youtube;

pub use soundcloud::resolve_soundcloud_stream;
pub use youtube::resolve_youtube_stream;

use crate::{
    api::{
        errors::ApiError,
        state::{AppState, cache::CachedStream},
    },
    types::SourcePlatform,
};

#[derive(Debug, Clone)]
pub struct ResolvedStreamInfo {
    pub stream_url: String,
    pub proxy_used: Option<String>,
}

pub async fn resolve_stream_url(
    state: &AppState,
    id: i64,
    url_param: Option<String>,
) -> Result<CachedStream, ApiError> {
    if let Some(cached) = state.cache.get_stream(id).await {
        return Ok(cached);
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
            .get_track(id)
            .and_then(|t| t.metadata.source_url.clone())
    };

    let resolved = match target_url
        .as_deref()
        .map(SourcePlatform::detect_from_url_str)
    {
        Some(SourcePlatform::YouTube) => {
            let u = target_url.as_deref().unwrap();
            resolve_youtube_stream(state, u).await?
        }
        _ => resolve_soundcloud_stream(state, id).await?,
    };

    state
        .cache
        .insert_stream(id, resolved.stream_url.clone(), resolved.proxy_used.clone())
        .await;

    state
        .cache
        .get_stream(id)
        .await
        .ok_or_else(|| ApiError::internal("Failed to cache stream"))
}

use super::ResolvedStreamInfo;
use crate::{
    api::{errors::ApiError, state::AppState},
    utils::soundcloud::{SoundcloudExt, resolve_soundcloud_stream_with_fallback},
};

pub async fn resolve_soundcloud_stream(
    state: &AppState,
    id: i64,
) -> Result<ResolvedStreamInfo, ApiError> {
    let cached_track = state.cache.get_soundcloud_track(id).await;

    if let Ok(url) = state
        .client
        .resolve_stream_url(cached_track.as_ref(), id)
        .await
    {
        let proxy = state
            .settings
            .read()
            .await
            .network
            .get_proxy_url()
            .map(String::from);
        return Ok(ResolvedStreamInfo {
            stream_url: url,
            proxy_used: proxy,
        });
    }

    let settings = state.settings.read().await.clone();
    let (url, _, proxy) = resolve_soundcloud_stream_with_fallback(&settings, id).await?;

    Ok(ResolvedStreamInfo {
        stream_url: url,
        proxy_used: proxy,
    })
}

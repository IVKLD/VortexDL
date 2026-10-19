use yt_audio_downloader::get_stream_info_with_client;

use super::ResolvedStreamInfo;
use crate::{
    api::{errors::ApiError, state::AppState},
    utils::proxy::race_proxies,
};

pub async fn resolve_youtube_stream(
    state: &AppState,
    target: &str,
) -> Result<ResolvedStreamInfo, ApiError> {
    let settings = state.settings.read().await.clone();
    let configured_proxy = settings.network.get_proxy_url().map(String::from);
    let active_fallback = state.cache.get_active_proxy().await;

    // 1. Fast path: configured proxy or direct connection
    let client = state
        .cache
        .get_or_create_client(configured_proxy.clone())
        .await;
    if let Ok(info) = get_stream_info_with_client(target, client, configured_proxy.clone()).await {
        return Ok(ResolvedStreamInfo {
            stream_url: info.stream_url,
            proxy_used: configured_proxy,
        });
    }

    // 2. Previously working fallback proxy before full racing
    if let Some(ref fallback_proxy) = active_fallback {
        let fallback_client = state
            .cache
            .get_or_create_client(Some(fallback_proxy.clone()))
            .await;
        if let Ok(info) =
            get_stream_info_with_client(target, fallback_client, Some(fallback_proxy.clone())).await
        {
            return Ok(ResolvedStreamInfo {
                stream_url: info.stream_url,
                proxy_used: Some(fallback_proxy.clone()),
            });
        }
    }

    // 3. Fallback: race configured proxies
    let target = target.to_string();
    let cache = state.cache.clone();
    let (info, used_proxy) = race_proxies(&settings, move |_, proxy| {
        let target = target.clone();
        let cache = cache.clone();
        async move {
            let client = cache.get_or_create_client(Some(proxy.clone())).await;
            let stream_info =
                get_stream_info_with_client(&target, client, Some(proxy.clone())).await?;
            Ok((stream_info, proxy))
        }
    })
    .await?;

    state.cache.set_active_proxy(Some(used_proxy.clone())).await;

    Ok(ResolvedStreamInfo {
        stream_url: info.stream_url,
        proxy_used: Some(used_proxy),
    })
}

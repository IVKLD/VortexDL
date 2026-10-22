use anyhow::Result;

use crate::{
    infra::net,
    providers::{ResolvedStreamInfo, cache::ProviderCache},
    settings::SettingsManager,
};

pub async fn resolve_youtube_stream(
    settings_mgr: &SettingsManager,
    cache: &ProviderCache,
    target: &str,
) -> Result<ResolvedStreamInfo> {
    let settings = settings_mgr.read().await.clone();
    let configured_proxy = settings.network.get_proxy_url().map(String::from);
    let active_fallback = cache.active_proxy();

    // 1. Fast path: configured proxy or direct connection
    let client = cache.get_or_create_client(configured_proxy.clone());
    if let Ok(info) =
        yt_audio_downloader::get_stream_info_with_client(target, client, configured_proxy.clone())
            .await
    {
        return Ok(ResolvedStreamInfo {
            stream_url: info.stream_url,
            proxy_used: configured_proxy,
        });
    }

    // 2. Previously working fallback proxy before full racing
    if let Some(ref fallback_proxy) = active_fallback {
        let fallback_client = cache.get_or_create_client(Some(fallback_proxy.clone()));
        if let Ok(info) = yt_audio_downloader::get_stream_info_with_client(
            target,
            fallback_client,
            Some(fallback_proxy.clone()),
        )
        .await
        {
            return Ok(ResolvedStreamInfo {
                stream_url: info.stream_url,
                proxy_used: Some(fallback_proxy.clone()),
            });
        }
    }

    // 3. Fallback: race configured proxies
    let target = target.to_string();
    let cache_clone = cache.clone();
    let proxies = settings.network.fallback_proxies.clone();
    let max_concurrent = settings.network.max_concurrent_proxies;

    let (info, used_proxy) = net::race_proxies(&proxies, max_concurrent, move |proxy| {
        let target = target.clone();
        let cache = cache_clone.clone();
        async move {
            let client = cache.get_or_create_client(Some(proxy.clone()));
            let stream_info = yt_audio_downloader::get_stream_info_with_client(
                &target,
                client,
                Some(proxy.clone()),
            )
            .await?;
            Ok((stream_info, proxy))
        }
    })
    .await?;

    cache.set_active_proxy(Some(used_proxy.clone()));

    Ok(ResolvedStreamInfo {
        stream_url: info.stream_url,
        proxy_used: Some(used_proxy),
    })
}

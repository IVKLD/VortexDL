use crate::{
    infra::net,
    providers::{ProviderError, ResolvedStreamInfo, youtube::YouTubeProvider},
};

impl YouTubeProvider {
    pub async fn resolve_stream(&self, target: &str) -> Result<ResolvedStreamInfo, ProviderError> {
        let settings = self.settings.read().await.clone();
        let configured_proxy = settings.network.get_proxy_url().map(String::from);
        let active_fallback = self.http_clients.active_fallback_proxy();

        let mut candidates = vec![configured_proxy];
        if let Some(fallback) = active_fallback {
            candidates.push(Some(fallback));
        }

        for proxy in candidates {
            let client = self.http_clients.get_or_create(proxy.clone());
            if let Ok(info) =
                yt_audio_downloader::get_stream_info_with_client(target, client, proxy.clone())
                    .await
            {
                return Ok(ResolvedStreamInfo {
                    stream_url: info.stream_url,
                    proxy_used: proxy,
                });
            }
        }

        let target = target.to_string();
        let http_clients = self.http_clients.clone();
        let proxies = settings.network.fallback_proxies.clone();
        let max_concurrent = settings.network.max_concurrent_proxies;

        let (info, used_proxy) = net::race_proxies(&proxies, max_concurrent, move |proxy| {
            let target = target.clone();
            let clients = http_clients.clone();
            async move {
                let client = clients.get_or_create(Some(proxy.clone()));
                let stream_info = yt_audio_downloader::get_stream_info_with_client(
                    &target,
                    client,
                    Some(proxy.clone()),
                )
                .await?;
                Ok((stream_info, proxy))
            }
        })
        .await
        .map_err(ProviderError::Other)?;

        self.http_clients
            .set_active_fallback_proxy(Some(used_proxy.clone()));

        Ok(ResolvedStreamInfo {
            stream_url: info.stream_url,
            proxy_used: Some(used_proxy),
        })
    }
}

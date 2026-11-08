use yt_audio_downloader::AudioStreamResponse;

use crate::{
    infra::net,
    providers::{
        ProviderError,
        youtube::{YouTubeProvider, client::ClientBuilder},
    },
};

impl YouTubeProvider {
    pub async fn resolve_stream(
        &self,
        target: &str,
    ) -> Result<(AudioStreamResponse, Option<String>), ProviderError> {
        match self.client.resolve_stream(target).await {
            Ok(info) => Ok((info, self.client.current_proxy_url().await)),
            Err(direct_err) => {
                let settings = self.settings.read().await.clone();
                if !settings.network.use_proxy || settings.network.fallback_proxies.is_empty() {
                    return Err(ProviderError::from(direct_err));
                }

                let target = target.to_string();
                let settings_clone = settings.clone();

                let (info, used_proxy) = net::race_proxies(
                    &settings.network.fallback_proxies,
                    settings.network.max_concurrent_proxies,
                    move |proxy| {
                        let target = target.clone();
                        let settings = settings_clone.clone();
                        async move {
                            let client = ClientBuilder::new(&settings)
                                .with_proxy(Some(&proxy))
                                .build()
                                .await?;
                            let stream_info = client.resolve_stream(&target).await?;
                            Ok((stream_info, proxy))
                        }
                    },
                )
                .await?;

                self.http_clients
                    .set_active_fallback_proxy(Some(used_proxy.clone()));

                Ok((info, Some(used_proxy)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_resolve_youtube() {
        let client = yt_audio_downloader::Client::new().unwrap();
        let res = client
            .resolve_stream("dQw4w9WgXcQ")
            .await
            .expect("resolve stream failed");
        assert!(!res.stream_url.is_empty(), "Stream URL should not be empty");
    }
}

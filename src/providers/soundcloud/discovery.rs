use soundcloud_rs::Client;
use url::Url;

use crate::{
    domain::DiscoveredTrack,
    infra::net,
    providers::{
        ProviderError,
        soundcloud::{SoundCloudProvider, client::ClientBuilder},
    },
};

impl SoundCloudProvider {
    pub async fn discover(&self, url: &Url) -> Result<Vec<DiscoveredTrack>, ProviderError> {
        tracing::info!("Resolving SoundCloud URL: {url}");

        match self.discover_with_client(url, &self.client).await {
            Ok(tracks) => Ok(tracks),
            Err(e) => {
                tracing::debug!("Direct discovery failed: {e}. Trying fallback proxies...");
                let settings = self.settings.read().await.clone();

                if !settings.network.use_proxy || settings.network.fallback_proxies.is_empty() {
                    return Err(e);
                }

                let settings_clone = settings.clone();
                let provider = self.clone();
                let url = url.clone();

                match net::race_proxies(
                    &settings.network.fallback_proxies,
                    settings.network.max_concurrent_proxies,
                    move |proxy| {
                        let settings = settings_clone.clone();
                        let provider = provider.clone();
                        let url = url.clone();

                        async move {
                            let proxied_client = ClientBuilder::new(&settings)
                                .with_proxy(Some(&proxy))
                                .build()
                                .await?;
                            provider
                                .discover_with_client(&url, &proxied_client)
                                .await
                                .map_err(Into::into)
                        }
                    },
                )
                .await
                {
                    Ok(tracks) => Ok(tracks),
                    Err(_) => Err(e),
                }
            }
        }
    }

    async fn discover_with_client(
        &self,
        url: &Url,
        client: &Client,
    ) -> Result<Vec<DiscoveredTrack>, ProviderError> {
        let tracks = client.resolve_to_tracks(url.as_str()).await?;
        Ok(tracks
            .into_iter()
            .filter_map(DiscoveredTrack::from_track)
            .collect())
    }
}

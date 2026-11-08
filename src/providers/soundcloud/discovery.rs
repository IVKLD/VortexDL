use soundcloud_rs::Client;
use url::Url;

use crate::{
    domain::DiscoveredTrack,
    infra::net,
    providers::{
        ProviderError,
        soundcloud::{SoundCloudProvider, client::ClientBuilder},
    },
    settings::UserSettings,
};

impl SoundCloudProvider {
    pub async fn discover(&self, url: &Url) -> Result<Vec<DiscoveredTrack>, ProviderError> {
        tracing::info!("Resolving SoundCloud URL: {url}");

        match self.discover_with_client(url, &self.client).await {
            Ok(tracks) => Ok(tracks),
            Err(direct_err) => {
                let settings = self.settings.read().await.clone();
                if !settings.network.use_proxy || settings.network.fallback_proxies.is_empty() {
                    return Err(direct_err);
                }

                self.race_discovery(&settings, url).await
            }
        }
    }

    async fn race_discovery(
        &self,
        settings: &UserSettings,
        url: &Url,
    ) -> Result<Vec<DiscoveredTrack>, ProviderError> {
        let settings_clone = settings.clone();
        let provider = self.clone();
        let target_url = url.clone();

        net::race_proxies(
            &settings.network.fallback_proxies,
            settings.network.max_concurrent_proxies,
            move |proxy| {
                let settings = settings_clone.clone();
                let provider = provider.clone();
                let url = target_url.clone();

                async move {
                    let client = ClientBuilder::new(&settings)
                        .with_proxy(Some(&proxy))
                        .build()
                        .await?;
                    let tracks = provider.discover_with_client(&url, &client).await?;
                    Ok(tracks)
                }
            },
        )
        .await
        .map_err(ProviderError::from)
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

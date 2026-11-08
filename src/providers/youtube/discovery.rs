use url::Url;
use yt_audio_downloader::Client;

use crate::{
    domain::DiscoveredTrack,
    infra::net,
    providers::{
        ProviderError,
        youtube::{YouTubeProvider, client::ClientBuilder},
    },
    settings::UserSettings,
};

impl YouTubeProvider {
    pub async fn discover(&self, url: &Url) -> Result<Vec<DiscoveredTrack>, ProviderError> {
        tracing::info!("Resolving YouTube URL: {url}");

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
        let url_str = url.as_str();

        if let Some(playlist_id) = yt_audio_downloader::extract_playlist_id(url_str) {
            let metadata_list = client.fetch_playlist(&playlist_id).await?;
            Ok(metadata_list.iter().map(DiscoveredTrack::from).collect())
        } else {
            let meta = client.fetch_metadata(url_str).await?;
            Ok(vec![DiscoveredTrack::from(&meta)])
        }
    }
}

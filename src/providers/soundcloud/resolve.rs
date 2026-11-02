use std::time::Duration;

use anyhow::anyhow;
use soundcloud_rs::StreamType;
use tokio::task::JoinHandle;
use url::Url;

use crate::providers::{
    ProviderError, ResolvedStreamInfo,
    soundcloud::{
        SoundCloudProvider,
        client::{ClientBuilder, SoundcloudExt},
    },
};

#[derive(Debug, Clone)]
pub enum StreamSource {
    Progressive {
        url: String,
        proxy_url: Option<String>,
    },
    Hls {
        url: String,
        proxy_url: Option<String>,
    },
}

impl StreamSource {
    pub fn new(stream_type: StreamType, url: String, proxy_url: Option<String>) -> Self {
        match stream_type {
            StreamType::Hls => Self::Hls { url, proxy_url },
            _ => Self::Progressive { url, proxy_url },
        }
    }

    pub fn url(&self) -> &str {
        match self {
            Self::Progressive { url, .. } | Self::Hls { url, .. } => url,
        }
    }
}

impl SoundCloudProvider {
    pub async fn resolve_stream(
        &self,
        cached_track: Option<&soundcloud_rs::Track>,
        id: i64,
    ) -> Result<ResolvedStreamInfo, ProviderError> {
        if let Ok(url) = self.client.resolve_stream_url(cached_track, id).await {
            let proxy = self
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

        let settings = self.settings.read().await.clone();
        let (url, _, proxy) = ClientBuilder::race_stream(&settings, id).await?;

        Ok(ResolvedStreamInfo {
            stream_url: url,
            proxy_used: proxy,
        })
    }

    pub async fn resolve_stream_source(&self, id: i64) -> Result<StreamSource, ProviderError> {
        match self.client.resolve_stream(id).await {
            Ok((url, stype)) => Ok(StreamSource::new(stype, url, self.client.proxy_url.clone())),
            Err(direct_err) => {
                tracing::debug!("Direct stream resolution failed for track {id}: {direct_err}");

                let settings = self.settings.read().await.clone();
                if !settings.network.use_proxy || settings.network.fallback_proxies.is_empty() {
                    return Err(direct_err);
                }

                let (url, stype, proxy_url) = ClientBuilder::race_stream(&settings, id)
                    .await
                    .map_err(|e| {
                        tracing::error!("All fallback proxies failed resolving track {id}: {e}");
                        ProviderError::Other(anyhow!("All fallback proxies failed: {e}"))
                    })?;

                Ok(StreamSource::new(stype, url, proxy_url))
            }
        }
    }

    pub fn spawn_artwork_fetch(
        &self,
        artwork_url: Option<&Url>,
    ) -> Option<JoinHandle<Option<Vec<u8>>>> {
        let url = artwork_url?.clone();
        let settings = self.settings.clone();
        let http_clients = self.http_clients.clone();
        Some(tokio::spawn(async move {
            let proxy_url = settings
                .read()
                .await
                .network
                .get_proxy_url()
                .map(String::from);
            let client = http_clients.get_or_create(proxy_url);
            let resp = client
                .get(url)
                .timeout(Duration::from_secs(5))
                .send()
                .await
                .ok()?;
            resp.bytes().await.ok().map(|b| b.to_vec())
        }))
    }
}

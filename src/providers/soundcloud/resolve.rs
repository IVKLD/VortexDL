use std::time::Duration;

use soundcloud_rs::{StreamSource, StreamType, Track};
use tokio::task::JoinHandle;
use url::Url;

use crate::{
    infra::net,
    providers::{
        ProviderError,
        soundcloud::{ClientBuilder, SoundCloudProvider},
    },
    settings::UserSettings,
};

impl SoundCloudProvider {
    pub async fn resolve_stream(
        &self,
        cached_track: Option<&Track>,
        id: i64,
    ) -> Result<StreamSource, ProviderError> {
        let direct = match cached_track {
            Some(t) => self.client.resolve_stream_from_track(t).await,
            None => self.client.resolve_stream(id).await,
        };

        match direct {
            Ok((url, stream_type)) => Ok(StreamSource {
                url,
                stream_type,
                proxy_url: self.client.proxy_url.clone(),
            }),
            Err(direct_err) => {
                let direct_err = ProviderError::from(direct_err);
                let settings = self.settings.read().await.clone();
                if !settings.network.use_proxy || settings.network.fallback_proxies.is_empty() {
                    return Err(direct_err);
                }

                let (url, stream_type, proxy_url) = race_stream(&settings, id).await?;
                Ok(StreamSource {
                    url,
                    stream_type,
                    proxy_url,
                })
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

async fn race_stream(
    settings: &UserSettings,
    id: i64,
) -> Result<(String, StreamType, Option<String>), ProviderError> {
    let settings_clone = settings.clone();
    let res = net::race_proxies(
        &settings.network.fallback_proxies,
        settings.network.max_concurrent_proxies,
        move |proxy| {
            let settings = settings_clone.clone();
            async move {
                let client = ClientBuilder::new(&settings)
                    .with_proxy(Some(&proxy))
                    .build()
                    .await?;
                let (url, stype) = client.resolve_stream(id).await?;
                Ok((url, stype, Some(proxy)))
            }
        },
    )
    .await?;
    Ok(res)
}

#[cfg(test)]
mod tests {
    use soundcloud_rs::ClientBuilder;

    use super::*;

    #[tokio::test]
    async fn test_resolve_soundcloud() {
        let client = ClientBuilder::new().build().await.unwrap();
        for track_id in [1736012493i64, 2233171517i64] {
            let track = client
                .get_track(track_id)
                .await
                .expect("track fetch failed");
            let (stream_url, stream_type) = client
                .resolve_stream_from_track(&track)
                .await
                .expect("resolve stream failed");
            assert!(!stream_url.is_empty(), "Stream URL should not be empty");
            assert_eq!(stream_type, StreamType::Hls);
        }
    }
}

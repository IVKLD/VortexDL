use std::time::Duration;

use anyhow::Result;
use soundcloud_rs::StreamType;
use tokio::task::JoinHandle;
use url::Url;

use crate::{
    downloader::Context,
    utils::{http::build_http_client, proxy::race_proxies, soundcloud, soundcloud::SoundcloudExt},
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
    pub fn url(&self) -> &str {
        match self {
            Self::Progressive { url, .. } | Self::Hls { url, .. } => url,
        }
    }
}

pub async fn resolve_stream_source(ctx: &Context, id: i64) -> Result<StreamSource> {
    match resolve_with_client(&ctx.client, id, ctx.client.proxy_url.as_deref()).await {
        Ok(proto) => Ok(proto),
        Err(direct_err) => {
            tracing::debug!("Direct stream resolution failed for track {id}: {direct_err}");

            let settings = ctx.settings.read().await.clone();
            if !settings.network.use_proxy || settings.network.fallback_proxies.is_empty() {
                return Err(direct_err);
            }

            race_proxies(&settings, move |s, proxy| async move {
                let client = soundcloud::ClientBuilder::new(&s)
                    .with_proxy(Some(&proxy))
                    .build()
                    .await?;
                resolve_with_client(&client, id, Some(&proxy)).await
            })
            .await
            .map_err(|e| {
                tracing::error!("All fallback proxies failed resolving track {id}: {e}");
                anyhow::anyhow!("All fallback proxies failed: {e}")
            })
        }
    }
}

async fn resolve_with_client(
    client: &soundcloud_rs::Client,
    id: i64,
    proxy_url: Option<&str>,
) -> Result<StreamSource> {
    let (url, stype) = client.resolve_stream(id).await?;
    let proxy_url = proxy_url.map(String::from);
    match stype {
        StreamType::Hls => Ok(StreamSource::Hls { url, proxy_url }),
        _ => Ok(StreamSource::Progressive { url, proxy_url }),
    }
}

pub fn spawn_artwork_fetch(
    ctx: &Context,
    artwork_url: Option<&Url>,
) -> Option<JoinHandle<Option<Vec<u8>>>> {
    let url = artwork_url?.clone();
    let ctx = ctx.clone();
    Some(tokio::spawn(async move {
        let proxy_url = ctx
            .settings
            .read()
            .await
            .network
            .get_proxy_url()
            .map(String::from);
        let client = build_http_client(proxy_url.as_deref(), 5, 10);
        let resp = client
            .get(url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .ok()?;
        resp.bytes().await.ok().map(|b| b.to_vec())
    }))
}

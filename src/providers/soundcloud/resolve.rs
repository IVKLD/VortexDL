use std::time::Duration;

use anyhow::{Result, anyhow};
use soundcloud_rs::{Client, StreamType};
use tokio::task::JoinHandle;
use url::Url;

use super::client::{SoundcloudExt, resolve_soundcloud_stream_with_fallback};
use crate::{
    downloader::DownloadContext,
    infra::net::build_http_client,
    providers::{ResolvedStreamInfo, cache::ProviderCache},
    settings::SettingsManager,
};

pub async fn resolve_soundcloud_stream(
    soundcloud: &Client,
    settings_mgr: &SettingsManager,
    cache: &ProviderCache,
    id: i64,
) -> Result<ResolvedStreamInfo> {
    let cached_track = cache.get_soundcloud_track(id);

    if let Ok(url) = soundcloud
        .resolve_stream_url(cached_track.as_ref(), id)
        .await
    {
        let proxy = settings_mgr
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

    let settings = settings_mgr.read().await.clone();
    let (url, _, proxy) = resolve_soundcloud_stream_with_fallback(&settings, id).await?;

    Ok(ResolvedStreamInfo {
        stream_url: url,
        proxy_used: proxy,
    })
}

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

pub async fn resolve_stream_source(ctx: &DownloadContext, id: i64) -> Result<StreamSource> {
    match resolve_with_client(&ctx.soundcloud, id, ctx.soundcloud.proxy_url.as_deref()).await {
        Ok(proto) => Ok(proto),
        Err(direct_err) => {
            tracing::debug!("Direct stream resolution failed for track {id}: {direct_err}");

            let settings = ctx.settings.read().await.clone();
            if !settings.network.use_proxy || settings.network.fallback_proxies.is_empty() {
                return Err(direct_err);
            }

            let (url, stype, proxy_url) = resolve_soundcloud_stream_with_fallback(&settings, id)
                .await
                .map_err(|e| {
                    tracing::error!("All fallback proxies failed resolving track {id}: {e}");
                    anyhow!("All fallback proxies failed: {e}")
                })?;

            Ok(StreamSource::new(stype, url, proxy_url))
        }
    }
}

async fn resolve_with_client(
    client: &Client,
    id: i64,
    proxy_url: Option<&str>,
) -> Result<StreamSource> {
    let (url, stype) = client.resolve_stream(id).await?;
    Ok(StreamSource::new(stype, url, proxy_url.map(String::from)))
}

pub fn spawn_artwork_fetch(
    ctx: &DownloadContext,
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

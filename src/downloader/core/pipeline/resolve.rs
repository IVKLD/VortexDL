use anyhow::Result;
use soundcloud_rs::{Identifier, StreamType};
use tokio::task::JoinHandle;

use crate::{
    downloader::Context,
    utils::{
        proxy::race_proxies,
        soundcloud::{fetch_artwork, init_client_with_settings},
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
    pub fn url(&self) -> &str {
        match self {
            Self::Progressive { url, .. } | Self::Hls { url, .. } => url,
        }
    }
}

pub async fn resolve_stream_source(ctx: &Context, id: i64) -> Result<StreamSource> {
    match resolve_with_client(&ctx.client, id, None).await {
        Ok(proto) => Ok(proto),
        Err(direct_err) => {
            tracing::debug!("Direct stream resolution failed for track {id}: {direct_err}");

            let settings = ctx.settings.read().await.clone();
            if settings.network.fallback_proxies.is_empty() {
                return Err(direct_err);
            }

            race_proxies(&settings, move |s, proxy| async move {
                let client = init_client_with_settings(&s, Some(&proxy)).await?;
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
    let track = client.get_track(&Identifier::Id(id)).await?;
    let proxy_str = proxy_url.map(String::from);
    let url = match client
        .resolve_stream_url_from_track(&track, Some(&StreamType::Hls))
        .await
    {
        Ok(url) => return Ok(StreamSource::Hls { url, proxy_url: proxy_str }),
        Err(_) => {
            client
                .resolve_stream_url_from_track(&track, Some(&StreamType::Progressive))
                .await?
        }
    };
    Ok(StreamSource::Progressive { url, proxy_url: proxy_str })
}

pub fn spawn_artwork_fetch(
    ctx: &Context,
    artwork_url: Option<&str>,
) -> Option<JoinHandle<Option<Vec<u8>>>> {
    let url = artwork_url?.to_string();
    let http = ctx.http.clone();
    Some(tokio::spawn(async move {
        fetch_artwork(&http, &url).await
    }))
}

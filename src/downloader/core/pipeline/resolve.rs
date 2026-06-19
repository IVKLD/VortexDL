use anyhow::Result;
use soundcloud_rs::{Client, Identifier, StreamType, Track};

use crate::{
    downloader::Context,
    utils::{proxy::race_proxies, soundcloud::init_client_with_settings},
};

#[derive(Debug, Clone)]
pub enum DownloadProtocol {
    Progressive {
        url: String,
        proxy_url: Option<String>,
    },
    Hls {
        url: String,
        proxy_url: Option<String>,
    },
}

impl DownloadProtocol {
    pub fn url(&self) -> &str {
        match self {
            Self::Progressive { url, .. } | Self::Hls { url, .. } => url,
        }
    }
}

pub async fn resolve_track_metadata(
    ctx: &Context,
    id: i64,
) -> Result<(Track, Identifier, DownloadProtocol)> {
    let sc_id = Identifier::Id(id);

    match try_resolve_with_client(&ctx.client, &sc_id, None).await {
        Ok((track, proto)) => Ok((track, sc_id, proto)),
        Err(direct_err) => {
            tracing::debug!("Direct stream resolution failed for track {id}: {direct_err}");

            let settings = ctx.settings.read().await.clone();

            race_proxies(&settings, move |s, proxy| async move {
                let client = init_client_with_settings(&s, Some(&proxy)).await?;
                let sc_id = Identifier::Id(id);
                let (track, proto) = try_resolve_with_client(&client, &sc_id, Some(&proxy)).await?;
                Ok((track, sc_id, proto))
            })
            .await
            .map_err(|e| {
                tracing::error!("All fallback proxies failed resolving track {id}: {e}");
                anyhow::anyhow!("All fallback proxies failed: {e}")
            })
        }
    }
}

async fn try_resolve_with_client(
    client: &Client,
    sc_id: &Identifier,
    proxy_url: Option<&str>,
) -> Result<(Track, DownloadProtocol)> {
    let track = client.get_track(sc_id).await?;
    let proxy_str = proxy_url.map(String::from);
    let protocol = match client.get_stream_url(sc_id, Some(&StreamType::Hls)).await {
        Ok(url) => DownloadProtocol::Hls {
            url,
            proxy_url: proxy_str,
        },
        Err(_) => {
            let url = client
                .get_stream_url(sc_id, Some(&StreamType::Progressive))
                .await?;
            DownloadProtocol::Progressive {
                url,
                proxy_url: proxy_str,
            }
        }
    };
    Ok((track, protocol))
}

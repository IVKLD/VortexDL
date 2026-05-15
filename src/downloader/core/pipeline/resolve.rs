use anyhow::{Result, anyhow};
use soundcloud_rs::{Identifier, StreamType, Track};

use crate::downloader::Context;

#[derive(Debug, Clone)]
pub enum DownloadProtocol {
    Progressive(String),
    Hls(String),
}

impl DownloadProtocol {
    pub fn url(&self) -> &str {
        match self {
            Self::Progressive(u) | Self::Hls(u) => u,
        }
    }
}

/// Resolves track metadata and available stream protocols.
pub async fn resolve_track_metadata(
    ctx: &Context,
    id: i64,
) -> Result<(Track, Identifier, DownloadProtocol)> {
    let sc_id = Identifier::Id(id);
    let track = ctx
        .client
        .get_track(&sc_id)
        .await
        .map_err(|e| anyhow!("Failed to resolve track {}: {}", id, e))?;

    let protocol = match ctx
        .client
        .get_stream_url(&sc_id, Some(&StreamType::Progressive))
        .await
    {
        Ok(url) => DownloadProtocol::Progressive(url),
        Err(_) => {
            let hls_url = ctx
                .client
                .get_stream_url(&sc_id, Some(&StreamType::Hls))
                .await
                .map_err(|e| anyhow!("Stream not available (Progressive & HLS failed): {}", e))?;
            DownloadProtocol::Hls(hls_url)
        }
    };

    Ok((track, sc_id, protocol))
}

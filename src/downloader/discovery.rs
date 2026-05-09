use std::sync::Arc;

use anyhow::Result;
use indicatif::ProgressBar;
use soundcloud_rs::Client;

use crate::{
    api::download_manager::{DownloadManager, ServerEvent},
    config::AppConfig,
    models::ResolveResponse,
    ui::create_standalone_spinner,
    utils::soundcloud::resolve_url,
};

pub mod likes;
pub mod playlist;
pub mod track;

pub use crate::downloader::discovery::{
    likes::fetch_likes, playlist::fetch_playlist, track::fetch_track,
};

pub struct DiscoveryContext<'a> {
    pub client: &'a Client,
    pub config: &'a AppConfig,
    pub dm: Option<&'a Arc<DownloadManager>>,
}

pub async fn resolve_with_feedback(
    ctx: &DiscoveryContext<'_>,
    url: &str,
    msg: &str,
) -> Result<ResolveResponse> {
    let pb = create_standalone_spinner(msg);

    if let Some(manager) = ctx.dm {
        manager.broadcast_event(ServerEvent::Message {
            message: msg.to_string(),
            level: "info".to_string(),
        });
    }

    let res = resolve_url(ctx.client, url).await;
    pb.finish_and_clear();

    res
}

pub fn show_feedback(ctx: &DiscoveryContext<'_>, msg: &str) -> ProgressBar {
    let pb = create_standalone_spinner(msg);

    if let Some(manager) = ctx.dm {
        manager.broadcast_event(ServerEvent::Message {
            message: msg.to_string(),
            level: "info".to_string(),
        });
    }

    pb
}

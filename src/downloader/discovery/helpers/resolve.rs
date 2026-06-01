use anyhow::Result;
use indicatif::ProgressBar;

use crate::{
    api::download_manager::ServerEvent,
    downloader::discovery::DiscoveryContext,
    models::{ResolveResponse, TrackLikesQuery, TrackLikesResponse},
    ui::create_standalone_spinner,
    utils::soundcloud::resolve_url,
};

pub async fn resolve_with_feedback(
    ctx: &DiscoveryContext<'_>,
    url: &str,
    msg: &str,
) -> Result<ResolveResponse> {
    let pb = show_feedback(ctx, msg);
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

pub async fn get_likes(
    client: &soundcloud_rs::Client,
    user_id: i64,
    offset: Option<&str>,
    limit: u32,
) -> Result<TrackLikesResponse> {
    let endpoint = format!("users/{user_id}/track_likes");
    let response = client
        .get(
            &endpoint,
            Some(&TrackLikesQuery {
                offset: offset.map(String::from),
                limit,
            }),
        )
        .await?;
    Ok(response)
}

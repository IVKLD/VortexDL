use anyhow::Result;
use indicatif::ProgressBar;

use crate::{
    api::download_manager::ServerEvent,
    downloader::discovery::DiscoveryContext,
    types::discovery::{AsUsername, TrackLikesQuery, TrackLikesResponse},
    ui::create_standalone_spinner,
    utils::filename::clean_title,
};

pub fn extract_artist<T: AsUsername>(user: Option<&T>) -> String {
    user.and_then(|u| u.username())
        .unwrap_or("Unknown")
        .to_string()
}

pub fn extract_title(title: Option<&str>) -> String {
    clean_title(title.unwrap_or("Unknown"))
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

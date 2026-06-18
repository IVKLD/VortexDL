use anyhow::Result;
use indicatif::ProgressBar;

use crate::{
    api::download_manager::ServerEvent,
    downloader::Context,
    types::discovery::{TrackLikesQuery, TrackLikesResponse},
    ui::create_standalone_spinner,
};

pub fn show_feedback(ctx: &Context, msg: &str) -> ProgressBar {
    let pb = create_standalone_spinner(msg);

    if let Some(manager) = &ctx.dm {
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
    Ok(client
        .get(
            &format!("users/{user_id}/track_likes"),
            Some(&TrackLikesQuery {
                offset: offset.map(String::from),
                limit,
            }),
        )
        .await?)
}

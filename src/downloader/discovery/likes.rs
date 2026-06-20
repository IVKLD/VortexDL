use anyhow::Result;
use soundcloud_rs::Client;
use url::Url;

use crate::downloader::{
    Context, DiscoveredTrack,
    discovery::{fetch_likes_page, init_progress_spinner},
};

pub async fn discover_liked_tracks(
    ctx: &Context,
    client: &Client,
    id: i64,
) -> Result<Vec<DiscoveredTrack>> {
    let mut offset: Option<String> = None;
    let pb = init_progress_spinner(ctx, "Fetching track list...");
    let mut tracks = Vec::new();
    let limit = ctx.settings.read().await.limit_per_page;

    loop {
        let res = fetch_likes_page(client, id, offset.as_deref(), limit).await?;
        if res.collection.is_empty() {
            break;
        }

        tracks.extend(
            res.collection
                .into_iter()
                .filter_map(|item| item.track.and_then(DiscoveredTrack::from_track)),
        );

        let Some(href) = res.next_href else {
            break;
        };
        offset = Url::parse(&href)?
            .query_pairs()
            .find(|(k, _)| k == "offset")
            .map(|(_, v)| v.into_owned());
    }

    pb.finish_and_clear();
    Ok(tracks)
}

use anyhow::Result;
use url::Url;

use crate::{
    downloader::{
        core::TrackDownload,
        discovery::{DiscoveryContext, resolve_with_feedback, show_feedback},
    },
    models::{TrackLikesQuery, TrackLikesResponse},
};

pub async fn fetch_likes(ctx: &DiscoveryContext<'_>, url: &str) -> Result<Vec<TrackDownload>> {
    let resolve_res = resolve_with_feedback(ctx, url, "Resolving user URL...").await?;

    let mut current_offset: Option<String> = None;
    let endpoint = format!("users/{}/track_likes", resolve_res.id);

    let pb = show_feedback(ctx, "Fetching track list...");

    let mut all_tracks = Vec::new();

    let limit = {
        let s = ctx.settings.read().await;
        s.limit_per_page
    };

    loop {
        let likes_query = TrackLikesQuery {
            offset: current_offset.clone(),
            limit,
        };

        let res: TrackLikesResponse = ctx.client.get(&endpoint, Some(&likes_query)).await?;

        if res.collection.is_empty() {
            break;
        }

        for item in res.collection {
            let id = item.track.id;
            let author = item
                .track
                .user
                .as_ref()
                .map(|u| u.username.as_str())
                .unwrap_or("Unknown");

            let title = item.track.title.as_str();
            let filename = format!("{} - {}", author, title);

            all_tracks.push(TrackDownload {
                id,
                filename,
                artwork_url: item.track.artwork_url.clone(),
            });
        }

        if let Some(next_href) = res.next_href {
            let parsed_url = Url::parse(&next_href)?;

            current_offset = parsed_url
                .query_pairs()
                .find(|(k, _)| k == "offset")
                .map(|(_, v)| v.into_owned());
        } else {
            break;
        }
    }

    pb.finish_and_clear();
    Ok(all_tracks)
}

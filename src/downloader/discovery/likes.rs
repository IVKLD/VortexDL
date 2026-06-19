use anyhow::Result;
use soundcloud_rs::Client;
use url::Url;

use crate::downloader::{
    Context, TrackDownload,
    discovery::{get_likes, show_feedback},
};

pub async fn fetch_likes(ctx: &Context, client: &Client, id: i64) -> Result<Vec<TrackDownload>> {
    let mut current_offset: Option<String> = None;
    let pb = show_feedback(ctx, "Fetching track list...");
    let mut all_tracks = Vec::new();
    let limit = ctx.settings.read().await.limit_per_page;

    loop {
        let res = get_likes(client, id, current_offset.as_deref(), limit).await?;
        if res.collection.is_empty() {
            break;
        }

        for item in res.collection {
            if let Some(track) = item.track {
                all_tracks.push(TrackDownload::new(
                    track.id,
                    Some(&track.title),
                    track.user.as_ref(),
                    track.artwork_url,
                    Some(all_tracks.len() as u32),
                ));
            }
        }

        let Some(href) = res.next_href else {
            break;
        };
        current_offset = Url::parse(&href)?
            .query_pairs()
            .find(|(k, _)| k == "offset")
            .map(|(_, v)| v.into_owned());
    }

    pb.finish_and_clear();
    Ok(all_tracks)
}

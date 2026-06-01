use anyhow::Result;
use url::Url;

use crate::downloader::{
    TrackDownload,
    discovery::{DiscoveryContext, extract_artist, extract_title, get_likes, show_feedback},
};

pub async fn fetch_likes(ctx: &DiscoveryContext<'_>, id: i64) -> Result<Vec<TrackDownload>> {
    let mut current_offset: Option<String> = None;
    let pb = show_feedback(ctx, "Fetching track list...");
    let mut all_tracks = Vec::new();
    let limit = ctx.settings.read().await.limit_per_page;

    loop {
        let res = get_likes(ctx.client, id, current_offset.as_deref(), limit).await?;
        if res.collection.is_empty() {
            break;
        }

        for item in res.collection {
            all_tracks.push(TrackDownload {
                id: item.track.id,
                title: extract_title(Some(&item.track.title)),
                artist: extract_artist(item.track.user.as_ref()),
                artwork_url: item.track.artwork_url,
                position: Some(all_tracks.len() as u32),
            });
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

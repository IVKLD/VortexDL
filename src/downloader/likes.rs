use colored::Colorize;
use soundcloud_rs::Client;
use std::collections::HashSet;
use std::error::Error;
use url::Url;

use crate::config::AppConfig;
use crate::models::{ResolveQuery, ResolveResponse, TrackLikesQuery, TrackLikesResponse};
use crate::storage::MusicStorage;
use super::core::process_tracks_concurrently;

pub async fn download_likes(
    storage: &MusicStorage,
    client: &Client,
    url_str: &str,
    output: &str,
    _config: &AppConfig,
) -> Result<HashSet<i64>, Box<dyn Error>> {
    println!("{} Resolving user URL...", "[INFO]".blue().bold());
    let resolve_res: ResolveResponse = client
        .get(
            "resolve",
            Some(&ResolveQuery {
                url: Some(url_str.to_string()),
            }),
        )
        .await?;

    let mut current_offset: Option<String> = None;
    let endpoint = format!("users/{}/track_likes", resolve_res.id);

    println!("{} Fetching track list...", "[INFO]".blue().bold());
    let mut all_tracks = Vec::new();
    let mut track_ids = HashSet::new();

    loop {
        let likes_res: TrackLikesResponse = client
            .get(
                &endpoint,
                Some(&TrackLikesQuery {
                    offset: current_offset.clone(),
                    limit: _config.limit_per_page,
                }),
            )
            .await?;

        if likes_res.collection.is_empty() {
            break;
        }

        for item in likes_res.collection {
            track_ids.insert(item.track.id);

            if storage.tracks.contains_key(&item.track.id) {
                println!(
                    "{} {}",
                    "[SKIP]".yellow().bold(),
                    &item.track.title
                );
            }

            all_tracks.push((item.track.id, item.track.title));
        }

        if let Some(next_href) = likes_res.next_href {
            current_offset = Url::parse(&next_href)?
                .query_pairs()
                .find(|(k, _)| k == "offset")
                .map(|(_, v)| v.into_owned());
        } else {
            break;
        }
    }

    process_tracks_concurrently(storage, client, all_tracks, output).await;

    Ok(track_ids)
}

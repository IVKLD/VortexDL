use std::sync::Arc;
use tokio::sync::RwLock;
use soundcloud_rs::Client;
use std::collections::HashSet;
use url::Url;
use colored::Colorize;

use super::core::process_tracks_concurrently;
use crate::config::AppConfig;
use crate::downloader::utils::ensure_likes_suffix;
use crate::models::{ResolveQuery, ResolveResponse, TrackLikesQuery, TrackLikesResponse};
use crate::storage::MusicStorage;
use crate::api::download_manager::DownloadManager;

pub async fn download_likes(
    storage: Arc<RwLock<MusicStorage>>,
    client: &Arc<Client>,
    url_str: &str,
    output: &str,
    config: Arc<AppConfig>,
    download_manager: Option<Arc<DownloadManager>>,
) -> crate::models::Result<HashSet<i64>> {
    println!("{} Resolving user URL...", "[INFO]".blue().bold());
    if let Some(ref dm) = download_manager {
        dm.broadcast_event(crate::api::download_manager::ServerEvent::Message {
            message: "Resolving user URL...".to_string(),
            level: "info".to_string(),
        });
    }

    let resolve_res: ResolveResponse = client
        .get(
            "resolve",
            Some(&ResolveQuery {
                url: Some(ensure_likes_suffix(url_str)),
            }),
        )
        .await?;

    let mut current_offset: Option<String> = None;
    let endpoint = format!("users/{}/track_likes", resolve_res.id);

    println!("{} Fetching track list...", "[INFO]".blue().bold());
    if let Some(ref dm) = download_manager {
        dm.broadcast_event(crate::api::download_manager::ServerEvent::Message {
            message: "Fetching track list...".to_string(),
            level: "info".to_string(),
        });
    }
    let mut all_tracks: Vec<(i64, String, Option<String>)> = Vec::new();
    let mut track_ids = HashSet::new();

    loop {
        let likes_res: TrackLikesResponse = client
            .get(
                &endpoint,
                Some(&TrackLikesQuery {
                    offset: current_offset.clone(),
                    limit: config.limit_per_page,
                }),
            )
            .await?;

        if likes_res.collection.is_empty() {
            break;
        }

        {
            let storage_read = storage.read().await;
            for item in likes_res.collection {
                track_ids.insert(item.track.id);

                if !storage_read.tracks.contains_key(&item.track.id) {
                    all_tracks.push((item.track.id, item.track.title.clone(), item.track.artwork_url.clone()));
                } else {
                    println!("{} Skipping: {}", "[SKIP]".yellow().bold(), item.track.title);
                }
            }
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

    if let Some(ref dm) = download_manager {
        for (id, filename, artwork_url) in &all_tracks {
            dm.add_task(*id, filename.clone(), artwork_url.clone()).await;
        }
    }

    process_tracks_concurrently(storage, client, all_tracks, output, download_manager).await;

    Ok(track_ids)
}

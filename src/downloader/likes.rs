use std::sync::Arc;
use tokio::sync::RwLock;
use soundcloud_rs::Client;
use std::collections::HashSet;
use url::Url;
use colored::Colorize;

use super::core::download_collection;
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
    dm: Option<Arc<DownloadManager>>,
) -> crate::models::Result<HashSet<i64>> {
    println!("{} Resolving user URL...", "[INFO]".blue().bold());
    
    if let Some(ref manager) = dm {
        manager.broadcast_event(crate::api::download_manager::ServerEvent::Message {
            message: "Resolving user URL...".to_string(),
            level: "info".to_string(),
        });
    }

    let query = ResolveQuery {
        url: Some(ensure_likes_suffix(url_str)),
    };

    let resolve_res: ResolveResponse = client
        .get("resolve", Some(&query))
        .await?;

    let mut current_offset: Option<String> = None;
    let endpoint = format!("users/{}/track_likes", resolve_res.id);

    println!("{} Fetching track list...", "[INFO]".blue().bold());
    
    if let Some(ref manager) = dm {
        manager.broadcast_event(crate::api::download_manager::ServerEvent::Message {
            message: "Fetching track list...".to_string(),
            level: "info".to_string(),
        });
    }
    
    let mut all_tracks = Vec::new();
    let mut track_ids = HashSet::new();

    loop {
        let likes_query = TrackLikesQuery {
            offset: current_offset.clone(),
            limit: config.limit_per_page,
        };

        let res: TrackLikesResponse = client
            .get(&endpoint, Some(&likes_query))
            .await?;

        if res.collection.is_empty() {
            break;
        }

        {
            let storage_read = storage.read().await;
            for item in res.collection {
                let id = item.track.id;
                track_ids.insert(id);

                if storage_read.tracks.contains_key(&id) {
                    println!("{} Skipping: {}", "[SKIP]".yellow().bold(), item.track.title);
                } else {
                    all_tracks.push((id, item.track.title.clone(), item.track.artwork_url.clone()));
                }
            }
        }

        if let Some(next_href) = res.next_href {
            let parsed_url = Url::parse(&next_href)?;
            
            current_offset = parsed_url.query_pairs()
                .find(|(k, _)| k == "offset")
                .map(|(_, v)| v.into_owned());
        } else {
            break;
        }
    }

    if let Some(ref manager) = dm {
        for (id, name, art) in &all_tracks {
            manager.add_task(*id, name.clone(), art.clone()).await;
        }
    }

    download_collection(storage, client, all_tracks, output, dm).await;
    
    Ok(track_ids)
}

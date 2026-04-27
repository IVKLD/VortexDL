use std::sync::Arc;
use tokio::sync::RwLock;
use soundcloud_rs::{Client, Identifier};
use std::collections::HashSet;
use colored::Colorize;

use crate::models::{ResolveQuery, ResolveResponse};
use crate::storage::MusicStorage;
use crate::api::download_manager::DownloadManager;
use super::core::process_tracks_concurrently;

pub async fn download_playlist(
    storage: Arc<RwLock<MusicStorage>>,
    client: &Arc<Client>,
    url: &str,
    output: &str,
    download_manager: Option<Arc<DownloadManager>>,
) -> crate::models::Result<HashSet<i64>> {
    println!("{} Resolving playlist URL...", "[INFO]".blue().bold());
    if let Some(ref dm) = download_manager {
        dm.broadcast_event(crate::api::download_manager::ServerEvent::Message {
            message: "Resolving playlist URL...".to_string(),
            level: "info".to_string(),
        });
    }
    
    let resolve_res: ResolveResponse = client
        .get(
            "resolve",
            Some(&ResolveQuery {
                url: Some(url.to_string()),
            }),
        )
        .await?;

    let playlist = client.get_playlist(&Identifier::Id(resolve_res.id)).await?;
    let mut track_ids = HashSet::new();

    let tracks_info: Vec<(i64, String, Option<String>)> = {
        let storage_read = storage.read().await;
        playlist
            .tracks
            .ok_or("No tracks found")?
            .into_iter()
            .filter_map(|t| {
                let id = t.id?;
                track_ids.insert(id);

                let author = t
                    .user
                    .as_ref()
                    .and_then(|u| u.username.as_deref())
                    .unwrap_or("Unknown");
                let title = t.title.as_deref().unwrap_or("Unknown");
                let full_name = format!("{} - {}", author, title);
                let artwork_url = t.artwork_url.clone();

                if storage_read.tracks.contains_key(&id) {
                    println!("{} Skipping: {}", "[SKIP]".yellow().bold(), full_name);
                    return None;
                }

                Some((id, full_name, artwork_url))
            })
            .collect()
    };

    if let Some(ref dm) = download_manager {
        for (id, filename, artwork_url) in &tracks_info {
            dm.add_task(*id, filename.clone(), artwork_url.clone()).await;
        }
    }

    process_tracks_concurrently(storage, client, tracks_info, output, download_manager).await;

    Ok(track_ids)
}

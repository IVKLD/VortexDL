use std::sync::Arc;
use tokio::sync::RwLock;
use soundcloud_rs::{Client, Identifier};
use std::collections::HashSet;
use colored::Colorize;

use crate::models::{ResolveQuery, ResolveResponse};
use crate::storage::MusicStorage;
use crate::api::download_manager::DownloadManager;
use super::core::download_collection;

pub async fn download_playlist(
    storage: Arc<RwLock<MusicStorage>>,
    client: &Arc<Client>,
    url: &str,
    output: &str,
    dm: Option<Arc<DownloadManager>>,
) -> anyhow::Result<HashSet<i64>> {
    println!("{} Resolving playlist URL...", "[INFO]".blue().bold());
    
    if let Some(ref manager) = dm {
        let msg = "Resolving playlist URL...".to_string();
        manager.broadcast_event(crate::api::download_manager::ServerEvent::Message {
            message: msg,
            level: "info".to_string(),
        });
    }
    
    let query = ResolveQuery {
        url: Some(url.to_string()),
    };

    let resolve_res: ResolveResponse = client
        .get("resolve", Some(&query))
        .await?;

    let playlist_id = Identifier::Id(resolve_res.id);
    let playlist = client.get_playlist(&playlist_id).await?;
    
    let mut track_ids = HashSet::new();
    let mut tracks_info = Vec::new();

    {
        let storage_read = storage.read().await;
        let tracks = playlist.tracks.ok_or_else(|| anyhow::anyhow!("No tracks found in playlist"))?;

        for track in tracks {
            let id = match track.id {
                Some(id) => id,
                None => continue,
            };
            
            track_ids.insert(id);

            let author = track.user
                .as_ref()
                .and_then(|u| u.username.as_deref())
                .unwrap_or("Unknown");
            
            let title = track.title.as_deref().unwrap_or("Unknown");
            let full_name = format!("{} - {}", author, title);
            let artwork_url = track.artwork_url.clone();

            if storage_read.tracks.contains_key(&id) {
                println!("{} Skipping: {}", "[SKIP]".yellow().bold(), full_name);
            } else {
                tracks_info.push((id, full_name, artwork_url));
            }
        }
    }

    if let Some(ref manager) = dm {
        for (id, name, art) in &tracks_info {
            manager.add_task(*id, name.clone(), art.clone()).await;
        }
    }

    download_collection(storage, client, tracks_info, output, dm).await;
    
    Ok(track_ids)
}

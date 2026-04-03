use colored::Colorize;
use soundcloud_rs::{Client, Identifier};
use std::collections::HashSet;
use std::error::Error;

use crate::models::{ResolveQuery, ResolveResponse};
use crate::storage::MusicStorage;
use super::core::process_tracks_concurrently;

pub async fn download_playlist(
    storage: &mut MusicStorage,
    client: &Client,
    url: &str,
    output: &str,
) -> Result<HashSet<i64>, Box<dyn Error>> {
    println!("{} Resolving playlist URL...", "[INFO]".blue().bold());
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

    let tracks_info: Vec<(i64, String)> = playlist
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

            if storage.tracks.contains_key(&id) {
                println!(
                    "{} Skipping: {}",
                    "[SKIP]".yellow().bold(),
                    title
                );
            }

            Some((id, format!("{} - {}", author, title)))
        })
        .collect();

    process_tracks_concurrently(storage, client, tracks_info, output).await;

    Ok(track_ids)
}

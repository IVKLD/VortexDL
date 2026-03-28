use colored::Colorize;
use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use soundcloud_rs::{Client, Identifier, StreamType};
use std::error::Error;
use std::str;
use std::time::Duration;
use url::Url;

use crate::config::AppConfig;
use crate::models::{ResolveQuery, ResolveResponse, TrackLikesQuery, TrackLikesResponse};
use crate::storage::MusicStorage;
use crate::utils::audio_file_manipulator::set_audio_custom_field;

pub async fn download_playlist(
    storage: &mut MusicStorage,
    client: &Client,
    url: &str,
    output: &str,
) -> Result<(), Box<dyn Error>> {
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
    let tracks_info: Vec<(i64, String)> = playlist
        .tracks
        .ok_or("No tracks found")?
        .into_iter()
        .filter_map(|t| {
            let id = t.id?;
            let author = t
                .user
                .and_then(|u| u.username)
                .unwrap_or_else(|| "Unknown".to_string());
            let title = t.title.unwrap_or_else(|| "Unknown".to_string());

            Some((id, format!("{} - {}", author, title)))
        })
        .collect();

    process_tracks_concurrently(storage, client, tracks_info, output).await;
    Ok(())
}

pub async fn download_likes(
    storage: &MusicStorage,
    client: &Client,
    url_str: &str,
    output: &str,
    _config: &AppConfig,
) -> Result<(), Box<dyn Error>> {
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
            if storage.tracks.contains_key(&item.track.id) {
                println!(
                    "{} Skipping: {}",
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

    Ok(())
}

async fn process_tracks_concurrently(
    storage: &MusicStorage,
    client: &Client,
    tracks: Vec<(i64, String)>,
    output_dir: &str,
) {
    if tracks.is_empty() {
        println!("{} No tracks found.", "[INFO]".yellow().bold());
        return;
    }

    let mut to_download = Vec::new();
    for (id, mut filename) in tracks {
        filename = filename
            .replace("/", "_")
            .replace("\\", "_")
            .replace(":", "_")
            .replace("?", "_")
            .replace("\"", "_")
            .replace("<", "_")
            .replace(">", "_")
            .replace("|", "_");

        if !storage.tracks.contains_key(&id) {
            to_download.push((id, filename));
        }
    }

    let total_tracks = to_download.len() as u64;
    println!(
        "{} Tracks to download: {}",
        "[INFO]".blue().bold(),
        total_tracks
    );

    let pb = ProgressBar::new(total_tracks);
    pb.enable_steady_tick(Duration::from_millis(500));

    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.yellow}[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    let concurrency_limit = 1;

    stream::iter(to_download)
        .for_each_concurrent(concurrency_limit, |(id, filename)| {
            let pb = pb.clone();
            async move {
                perform_track_download(TrackDownloadTask {
                    client,
                    id,
                    filename,
                    output_dir,
                    progress_bar: pb,
                })
                .await;
            }
        })
        .await;

    pb.finish_and_clear();
    println!("{} All tasks processed.", "[SUCCESS]".green().bold());
}

struct TrackDownloadTask<'a> {
    client: &'a Client,
    id: i64,
    filename: String,
    output_dir: &'a str,
    progress_bar: ProgressBar,
}

async fn perform_track_download(task: TrackDownloadTask<'_>) {
    let TrackDownloadTask {
        client,
        id,
        filename,
        output_dir,
        progress_bar,
    } = task;

    progress_bar.set_message(format!("Downloading: {}", filename));

    let result = match client.get_track(&Identifier::Id(id)).await {
        Ok(track) => {
            let transcodings = track
                .media
                .as_ref()
                .expect("Missing media")
                .transcodings
                .as_ref()
                .expect("Missing transcodings");

            let stream = transcodings
                .last()
                .and_then(|value| value.format.as_ref())
                .and_then(|value| value.protocol.as_ref());

            client
                .download_track(
                    &track,
                    &Identifier::Id(id),
                    stream,
                    Some(output_dir),
                    Some(&filename),
                )
                .await
        }
        Err(e) => Err(e),
    };

    if result.is_ok() {
        let file_path = format!("{}/{}.mp3", output_dir, filename);

        if let Err(e) = set_audio_custom_field(&file_path, "sc-identifier", &id.to_string()) {
            progress_bar.println(format!("Failed to set custom field: {:#?}", e));
        }

        progress_bar.println(format!("{} Done: {}", "[OK]".green().bold(), filename));
    } else {
        progress_bar.println(format!("{} Failed: {}", "[ERROR]".red().bold(), filename));

        if let Some(e) = result.err() {
            progress_bar.println(format!("Details: {:#?}", e));
        }
    }

    progress_bar.inc(1);
}

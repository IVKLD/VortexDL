use std::sync::Arc;
use tokio::sync::RwLock;
use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use soundcloud_rs::{Client, Identifier};
use std::time::Duration;
use std::path::PathBuf;
use colored::Colorize;

use crate::storage::MusicStorage;
use crate::models::{SC_IDENTIFIER, SC_ARTWORK_URL};
use crate::utils::audio_file_manipulator::set_audio_custom_field;
use crate::api::download_manager::{DownloadManager, DownloadStatus};
use super::utils::clean_filename;

pub async fn process_tracks_concurrently(
    storage: Arc<RwLock<MusicStorage>>,
    client: &Arc<Client>,
    tracks: Vec<(i64, String, Option<String>)>,
    output_dir: &str,
    download_manager: Option<Arc<DownloadManager>>,
) {
    if tracks.is_empty() {
        return;
    }

    let mut to_download = Vec::new();
    let mut skipped_count = 0;
    {
        let storage_read = storage.read().await;
        for (id, filename, artwork_url) in tracks {
            let cleaned_filename = clean_filename(&filename);
            if !storage_read.tracks.contains_key(&id) {
                to_download.push((id, cleaned_filename, artwork_url));
            } else {
                skipped_count += 1;
            }
        }
    }

    if skipped_count > 0 {
        println!("{} Skipped {} tracks (already in storage).", "[INFO]".blue().bold(), skipped_count);
    }

    if to_download.is_empty() {
        println!("{} Everything is already synced!", "[INFO]".blue().bold());
        return;
    }

    let total_tracks = to_download.len() as u64;
    
    println!("{} Starting download of {} tracks.", "[INFO]".blue().bold(), total_tracks);

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
        .for_each_concurrent(concurrency_limit, |(id, filename, artwork_url)| {
            let pb = pb.clone();
            let storage = storage.clone();
            let download_manager = download_manager.clone();
            let client = client.clone();
            let output_dir = output_dir.to_string();
            async move {
                perform_track_download(TrackDownloadTask {
                    client,
                    id,
                    filename,
                    artwork_url,
                    output_dir,
                    progress_bar: pb,
                    storage,
                    download_manager,
                })
                .await;
            }
        })
        .await;

    pb.finish_and_clear();
    println!("{} Synchronization complete. {} new tracks downloaded, {} tracks were already synced.", 
        "[SUCCESS]".green().bold(), total_tracks, skipped_count);
}

pub struct TrackDownloadTask {
    pub client: Arc<Client>,
    pub id: i64,
    pub filename: String,
    pub artwork_url: Option<String>,
    pub output_dir: String,
    pub progress_bar: ProgressBar,
    pub storage: Arc<RwLock<MusicStorage>>,
    pub download_manager: Option<Arc<DownloadManager>>,
}

pub async fn perform_track_download(task: TrackDownloadTask) {
    let TrackDownloadTask {
        client,
        id,
        filename,
        artwork_url,
        output_dir,
        progress_bar,
        storage,
        download_manager,
    } = task;

    if let Some(ref dm) = download_manager {
        dm.update_status(id, DownloadStatus::Downloading).await;
    }

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
                    Some(&output_dir),
                    Some(&filename),
                )
                .await
        }
        Err(e) => Err(e),
    };

    if result.is_ok() {
        let file_path = format!("{}/{}.mp3", output_dir, filename);
        let path_buf = PathBuf::from(&file_path);

        if let Err(e) = set_audio_custom_field(&file_path, SC_IDENTIFIER, &id.to_string()) {
            progress_bar.println(format!("Failed to set custom field: {:#?}", e));
        }

        if let Some(ref url) = artwork_url {
            if let Err(e) = set_audio_custom_field(&file_path, SC_ARTWORK_URL, url) {
                progress_bar.println(format!("Failed to set artwork URL: {:#?}", e));
            }

            let high_res_url = url.replace("-large.jpg", "-t500x500.jpg");
            match reqwest::get(&high_res_url).await {
                Ok(resp) => {
                    if let Ok(bytes) = resp.bytes().await {
                        if let Err(e) = crate::utils::audio_file_manipulator::set_artwork(&file_path, bytes.to_vec()) {
                            progress_bar.println(format!("Failed to set artwork: {:#?}", e));
                        }
                    }
                }
                Err(e) => progress_bar.println(format!("Failed to download artwork: {:#?}", e)),
            }
        }

        {
            let mut storage_write = storage.write().await;
            storage_write.update_track(id, path_buf, artwork_url);
        }

        if let Some(ref dm) = download_manager {
            dm.update_status(id, DownloadStatus::Finished).await;
        }

        progress_bar.println(format!("{} Done: {}", "[OK]".green().bold(), filename));
    } else {
        if let Some(ref dm) = download_manager {
            dm.update_status(id, DownloadStatus::Failed).await;
        }

        progress_bar.println(format!("{} Failed: {}", "[ERROR]".red().bold(), filename));

        if let Some(e) = result.err() {
            progress_bar.println(format!("Details: {:#?}", e));
        }
    }

    progress_bar.inc(1);
}

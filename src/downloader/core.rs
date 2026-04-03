use colored::Colorize;
use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use soundcloud_rs::{Client, Identifier};
use std::time::Duration;

use crate::storage::MusicStorage;
use crate::utils::audio_file_manipulator::set_audio_custom_field;
use super::utils::clean_filename;

pub async fn process_tracks_concurrently(
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
    for (id, filename) in tracks {
        let cleaned_filename = clean_filename(&filename);
        if !storage.tracks.contains_key(&id) {
            to_download.push((id, cleaned_filename));
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

pub struct TrackDownloadTask<'a> {
    pub client: &'a Client,
    pub id: i64,
    pub filename: String,
    pub output_dir: &'a str,
    pub progress_bar: ProgressBar,
}

pub async fn perform_track_download(task: TrackDownloadTask<'_>) {
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

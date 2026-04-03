use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use soundcloud_rs::{Client, Identifier};
use std::error::Error;

use crate::storage::MusicStorage;
use super::core::{perform_track_download, TrackDownloadTask};

pub async fn download_track(
    storage: &MusicStorage,
    client: &Client,
    id: i64,
    output: &str,
) -> Result<(), Box<dyn Error>> {
    let track = client.get_track(&Identifier::Id(id)).await?;
    let author = track
        .user
        .as_ref()
        .and_then(|u| u.username.as_deref())
        .unwrap_or("Unknown");
    let title = track.title.as_deref().unwrap_or("Unknown");
    let filename = format!("{} - {}", author, title);

    if storage.tracks.contains_key(&id) {
        println!("{} Skipping: {}", "[SKIP]".yellow().bold(), filename);
        return Ok(());
    }

    let pb = ProgressBar::new(1);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.yellow}[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap(),
    );

    perform_track_download(TrackDownloadTask {
        client,
        id,
        filename,
        output_dir: output,
        progress_bar: pb,
    })
    .await;

    Ok(())
}

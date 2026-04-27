use std::sync::Arc;
use tokio::sync::RwLock;
use indicatif::{ProgressBar, ProgressStyle};
use soundcloud_rs::{Client, Identifier};
use colored::Colorize;

use crate::storage::MusicStorage;
use crate::api::download_manager::DownloadManager;
use super::core::{perform_track_download, TrackDownloadTask};

pub async fn download_track(
    storage: Arc<RwLock<MusicStorage>>,
    client: &Arc<Client>,
    id: i64,
    output: &str,
    download_manager: Option<Arc<DownloadManager>>,
) -> crate::models::Result<()> {
    let track = client.get_track(&Identifier::Id(id)).await?;
    let author = track
        .user
        .as_ref()
        .and_then(|u| u.username.as_deref())
        .unwrap_or("Unknown");
    let title = track.title.as_deref().unwrap_or("Unknown");
    let filename = format!("{} - {}", author, title);
    let artwork_url = track.artwork_url.clone();

    if let Some(ref dm) = download_manager {
        dm.add_task(id, filename.clone(), artwork_url.clone()).await;
    }

    {
        let storage_read = storage.read().await;
        if storage_read.tracks.contains_key(&id) {
            println!("{} Skipping: {}", "[SKIP]".yellow().bold(), filename);
            return Ok(());
        }
    }

    let pb = ProgressBar::new(1);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.yellow}[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap(),
    );

    perform_track_download(TrackDownloadTask {
        client: client.clone(),
        id,
        filename,
        artwork_url,
        output_dir: output.to_string(),
        progress_bar: pb,
        storage,
        download_manager,
    })
    .await;

    Ok(())
}

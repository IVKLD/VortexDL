use anyhow::{Result, anyhow};
use tokio::fs;
use yt_audio_downloader::{AudioFormat, ProgressEvent, YoutubeAudioDownloader};

use crate::downloader::{Context, DownloadTask};

pub async fn download_youtube_track(
    context: &Context,
    task: &DownloadTask,
    url_or_id: &str,
) -> Result<()> {
    let output_dir = task
        .file_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));

    let dm = context.dm.clone();
    let track_id = task.track.id;

    let downloader = YoutubeAudioDownloader::new()
        .output_dir(output_dir)
        .format(AudioFormat::Mp3)
        .on_progress(move |event| {
            if let ProgressEvent::Downloading {
                bytes_downloaded,
                total_bytes,
                ..
            } = event
                && let Some(ref manager) = dm
            {
                manager.update_progress(track_id, bytes_downloaded, total_bytes.unwrap_or(0));
            }
        });

    let downloaded = downloader
        .download(url_or_id)
        .await
        .map_err(|e| anyhow!("YouTube download failed: {e}"))?;

    if downloaded.file_path != task.file_path {
        if task.file_path.exists() {
            let _ = fs::remove_file(&task.file_path).await;
        }
        fs::rename(&downloaded.file_path, &task.file_path).await?;
    }

    Ok(())
}

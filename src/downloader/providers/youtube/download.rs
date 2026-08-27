use anyhow::{Context, Result};
use yt_audio_downloader::{AudioFormat, ProgressEvent, YoutubeAudioDownloader};

use crate::downloader::{Context as AppContext, DownloadTask};

pub async fn download_youtube_track(
    context: &AppContext,
    task: &DownloadTask,
    url_or_id: &str,
) -> Result<()> {
    let dm = context.dm.clone();
    let track_id = task.track.id;

    YoutubeAudioDownloader::new()
        .output_file(&task.file_path)
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
        })
        .download(url_or_id)
        .await
        .context("YouTube download failed")?;

    Ok(())
}

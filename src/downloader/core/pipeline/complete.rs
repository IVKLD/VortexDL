use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use colored::Colorize;
use tokio::task::JoinHandle;

use super::artwork::ArtworkDataHandle;
use crate::{
    api::download_manager::DownloadStatus,
    downloader::{Context, core::pipeline::DownloadTask},
    storage::TrackData,
    utils::{
        filename::format_track_filename,
        metadata::{SaveTrackArgs, save_track_info},
    },
};

/// Handles successful download completion.
pub fn finalize_and_persist(
    ctx: Context,
    task: DownloadTask,
    artwork_handle: Option<ArtworkDataHandle>,
    source_url: String,
) -> JoinHandle<()> {
    task.pb.println(format!(
        "{} Done: {}",
        "[OK]".green().bold(),
        format_track_filename(&task.artist, &task.title)
    ));

    tokio::spawn(persist(ctx, task, artwork_handle, source_url))
}

/// Persists the downloaded track (DB update, metadata tagging).
async fn persist(
    ctx: Context,
    task: DownloadTask,
    artwork_handle: Option<ArtworkDataHandle>,
    source_url: String,
) {
    let artwork_data = if let Some(h) = artwork_handle {
        h.await.unwrap_or_default()
    } else {
        None
    };

    ctx.storage.write().await.update_track(
        task.id,
        TrackData {
            path: PathBuf::from(&task.file_path),
            artist: task.artist.clone(),
            title: task.title.clone(),
            artwork_url: task.artwork_url.clone(),
            source_url: Some(source_url.clone()),
            position: task.position,
        },
    );

    let id = task.id;
    let file_path = task.file_path.clone();
    let source_url_clone = source_url.clone();

    let _ = tokio::task::spawn_blocking(move || {
        let sc_id = task.id.to_string();
        let _ = save_track_info(SaveTrackArgs {
            path: &task.file_path,
            sc_id: &sc_id,
            title: &task.title,
            artist: &task.artist,
            artwork_url: task.artwork_url.as_deref(),
            source_url: Some(&source_url_clone),
            position: task.position,
            artwork_data,
        });
    })
    .await;

    if let Some(m) = ctx.dm {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let size = std::fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0);
        m.update_finished(id, "mp3".to_string(), now, Some(source_url), size)
            .await;
    }
}

/// Handles download failure.
pub async fn on_failure(ctx: &Context, task: &DownloadTask, e: anyhow::Error) {
    if let Some(m) = &ctx.dm {
        m.update_status(task.id, DownloadStatus::Failed).await;
    }
    task.pb.println(format!(
        "{} Failed: {} — {:#}",
        "[ERROR]".red().bold(),
        format_track_filename(&task.artist, &task.title),
        e
    ));
}

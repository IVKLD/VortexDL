use std::time::{SystemTime, UNIX_EPOCH};

use colored::Colorize;
use tokio::task::JoinHandle;

use crate::{
    downloader::{
        Context,
        core::pipeline::{DownloadTask, artwork::ArtworkDataHandle},
    },
    storage::TrackData,
    api::types::AudioFormat,
    utils::metadata::{SaveTrackArgs, save_track_info},
};

pub fn finalize_and_persist(
    ctx: Context,
    task: DownloadTask,
    artwork_handle: Option<ArtworkDataHandle>,
    source_url: String,
) -> JoinHandle<()> {
    task.pb
        .println(format!("{} {}", "[OK]".green().bold(), task.display_name()));

    tokio::spawn(persist(ctx, task, artwork_handle, source_url))
}

async fn persist(
    ctx: Context,
    task: DownloadTask,
    artwork_handle: Option<ArtworkDataHandle>,
    source_url: String,
) {
    let artwork_data = match artwork_handle {
        Some(h) => h.await.unwrap_or_default(),
        None => None,
    };

    let size = tokio::fs::metadata(&task.file_path)
        .await
        .map_or(0, |m| m.len());

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    ctx.storage.write().await.update_track(
        task.id,
        TrackData {
            path: task.file_path.clone(),
            artist: task.artist.clone(),
            title: task.title.clone(),
            artwork_url: task.artwork_url.clone(),
            source_url: Some(source_url.clone()),
            position: task.position,
            created_at: now,
            size,
        },
    );

    let id = task.id;

    if let Some(m) = ctx.dm {
        let format = AudioFormat::from_path(&task.file_path);
        m.update_finished(id, format, now, Some(source_url.clone()), size);
    }

    let _ = tokio::task::spawn_blocking(move || {
        let sc_id = task.id.to_string();
        let _ = save_track_info(SaveTrackArgs {
            path: &task.file_path,
            sc_id: &sc_id,
            title: &task.title,
            artist: &task.artist,
            artwork_url: task.artwork_url.as_deref(),
            source_url: Some(&source_url),
            position: task.position,
            artwork_data,
        });
    })
    .await;
}

pub async fn on_failure(ctx: &Context, task: &DownloadTask, e: anyhow::Error) {
    if let Some(m) = &ctx.dm {
        m.update_failed(task.id, format!("{:#}", e));
    }
    task.pb.println(format!(
        "{} {} — {:#}",
        "[ERROR]".red().bold(),
        task.display_name(),
        e
    ));
}

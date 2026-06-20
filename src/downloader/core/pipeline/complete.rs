use std::time::{SystemTime, UNIX_EPOCH};

use colored::Colorize;
use tokio::task::JoinHandle;

use crate::{
    api::types::AudioFormat,
    downloader::{Context, core::pipeline::DownloadTask},
    storage::LocalTrack,
    utils::metadata::{SaveTrackArgs, save_track_info},
};

pub async fn finalize(
    context: Context,
    task: DownloadTask,
    artwork_handle: Option<JoinHandle<Option<Vec<u8>>>>,
    source_url: String,
) {
    task.pb
        .println(format!("{} {}", "[OK]".green().bold(), task.display_name()));

    let artwork_data = match artwork_handle {
        Some(handle) => handle.await.unwrap_or_default(),
        None => None,
    };

    let size = tokio::fs::metadata(&task.file_path)
        .await
        .map_or(0, |metadata| metadata.len());

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    context.storage.write().await.update_track(
        task.id,
        LocalTrack {
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

    if let Some(manager) = context.dm {
        let format = AudioFormat::from_path(&task.file_path);
        manager.update_finished(task.id, format, now, Some(source_url.clone()), size);
    }

    let track_id = task.id;
    let res = tokio::task::spawn_blocking(move || {
        let sc_id = task.id.to_string();
        save_track_info(SaveTrackArgs {
            path: &task.file_path,
            sc_id: &sc_id,
            title: &task.title,
            artist: &task.artist,
            artwork_url: task.artwork_url.as_deref(),
            source_url: Some(&source_url),
            position: task.position,
            artwork_data,
        })
    })
    .await;

    match res {
        Err(err) => {
            tracing::error!(track_id, "Metadata task panicked or was cancelled: {err:#}");
        }
        Ok(Err(err)) => {
            tracing::warn!(track_id, "Failed to save track metadata: {err:#}");
        }
        _ => {}
    }
}

pub async fn fail(context: &Context, task: &DownloadTask, err: anyhow::Error) {
    if let Some(manager) = &context.dm {
        manager.update_failed(task.id, format!("{:#}", err));
    }
    task.pb.println(format!(
        "{} {} — {:#}",
        "[ERROR]".red().bold(),
        task.display_name(),
        err
    ));
}

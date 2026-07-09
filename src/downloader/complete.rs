use std::{
    collections::{HashMap, HashSet},
    time::SystemTime,
};

use colored::Colorize;
use tokio::task::JoinHandle;
use url::Url;

use super::DownloadTask;
use crate::{
    adb_device,
    api::types::AudioFormat,
    database::cache::{CachedMusicTrack, update_cached_tracks_batch},
    downloader::Context,
    storage::{LocalMusicTrack, metadata::{SaveTrackArgs, save_track_info}},
    utils::soundcloud,
};

pub async fn finalize_pipeline_sync(
    context: &Context,
    url: &Url,
    remote_ids: &HashSet<i64>,
) -> anyhow::Result<()> {
    context
        .storage
        .write()
        .await
        .sync_storage(url, remote_ids)
        .await?;

    adb_device::sync_connected(context.storage.clone(), context.settings.clone()).await;

    soundcloud::update_cached_client_id(&context.client, &context.settings).await;

    Ok(())
}

pub async fn finalize_single_track(
    context: Context,
    task: DownloadTask,
    artwork_handle: Option<JoinHandle<Option<Vec<u8>>>>,
) {
    task.pb
        .println(format!("{} {}", "[OK]".green().bold(), task.display_name()));
    task.pb.finish_and_clear();

    let artwork_data = match artwork_handle {
        Some(handle) => handle.await.unwrap_or_default(),
        None => None,
    };

    let file_metadata = tokio::fs::metadata(&task.file_path).await.ok();
    let size = file_metadata.as_ref().map_or(0, |m| m.len());

    let now = crate::utils::system_time_to_secs(SystemTime::now());

    let mtime = file_metadata
        .and_then(|m| m.modified().ok())
        .map(crate::utils::system_time_to_secs)
        .unwrap_or(now);

    let permalink_url = &task.track.permalink_url;
    let metadata = task.track.to_metadata(permalink_url.as_ref().map(|u| u.to_string()));

    context.storage.write().await.update_track(
        task.track.id,
        LocalMusicTrack {
            path: task.file_path.clone(),
            metadata: metadata.clone(),
            created_at: now,
            size,
        },
    );

    let cached_track = CachedMusicTrack {
        metadata,
        created_at: now,
        size,
        mtime,
    };

    let path_str = task.file_path.to_string_lossy().into_owned();
    let _ = tokio::task::spawn_blocking(move || {
        let mut to_update = HashMap::new();
        to_update.insert(path_str, cached_track);
        let _ = update_cached_tracks_batch(&to_update, &HashSet::new());
    })
    .await;

    if let Some(manager) = context.dm {
        let format = AudioFormat::from_path(&task.file_path);
        manager.update_finished(task.track.id, format, now, permalink_url.as_ref(), size);
    }

    let track_id = task.track.id;
    let res = tokio::task::spawn_blocking(move || {
        let sc_id = task.track.id.to_string();
        save_track_info(SaveTrackArgs {
            path: &task.file_path,
            sc_id: &sc_id,
            title: &task.track.title,
            artist: &task.track.artist,
            artwork_url: task.track.artwork_url.as_ref(),
            source_url: task.track.permalink_url.as_ref(),
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

pub async fn handle_track_failure(context: &Context, task: &DownloadTask, err: anyhow::Error) {
    let full_err_msg = format!("{:#}", err);
    if let Some(manager) = &context.dm {
        manager.update_failed(task.track.id, &full_err_msg);
    }

    let mut short_err = full_err_msg;
    if let Some(idx) = short_err.find("url (") {
        short_err.truncate(idx + 4);
        short_err.push_str("...)");
    } else if let Some(idx) = short_err.find("for url") {
        short_err.truncate(idx + 7);
        short_err.push_str(" (...)");
    }

    task.pb.println(format!(
        "{} {} — {}",
        "[ERROR]".red().bold(),
        task.display_name(),
        short_err
    ));
    task.pb.finish_and_clear();
}

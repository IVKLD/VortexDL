use std::{collections::HashSet, time::SystemTime};

use colored::Colorize;
use tokio::task::JoinHandle;
use url::Url;

use super::DownloadTask;
use crate::{
    adb_device,
    api::types::AudioFormat,
    database::cache::{CachedMusicTrack, update_cached_tracks_batch},
    downloader::Context,
    storage::{
        LocalMusicTrack,
        metadata::{SaveTrackArgs, save_track_info},
        sync::sync_url_ids,
    },
    utils::soundcloud,
};

pub async fn finalize_pipeline_sync(
    context: &Context,
    url: &Url,
    remote_ids: &HashSet<i64>,
) -> anyhow::Result<()> {
    sync_url_ids(url, remote_ids).await?;

    adb_device::sync_connected(context.storage.clone(), context.settings.clone()).await;

    soundcloud::update_cached_client_id(&context.client, &context.settings).await;

    Ok(())
}

pub async fn finalize_single_track(
    context: Context,
    task: DownloadTask,
    artwork_handle: Option<JoinHandle<Option<Vec<u8>>>>,
    pb: indicatif::ProgressBar,
) {
    pb.println(format!("{} {}", "[OK]".green().bold(), task.display_name()));

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

    let permalink_url = task.track.permalink_url.clone();
    let metadata = task
        .track
        .to_metadata(permalink_url.as_ref().map(|u| u.to_string()));

    context.storage.write().await.update_track(
        task.track.id,
        LocalMusicTrack {
            path: task.file_path.clone(),
            metadata: metadata.clone(),
            created_at: now,
            size,
        },
    );

    if let Some(ref manager) = context.dm {
        let format = AudioFormat::from_path(&task.file_path);
        manager.update_finished(task.track.id, format, now, permalink_url.as_ref(), size);
    }

    let track_id = task.track.id;
    let path_str = task.file_path.to_string_lossy().into_owned();
    let cached_track = CachedMusicTrack {
        metadata,
        created_at: now,
        size,
        mtime,
    };

    let cache_task = tokio::task::spawn_blocking(move || {
        let mut to_update = std::collections::HashMap::new();
        to_update.insert(path_str, cached_track);
        update_cached_tracks_batch(&to_update, &HashSet::new())
    });

    let sc_id = task.track.id.to_string();
    let file_path = task.file_path.clone();
    let title = task.track.title.clone();
    let artist = task.track.artist.clone();
    let artwork_url = task.track.artwork_url.clone();
    let source_url = task.track.permalink_url.clone();

    let meta_task = tokio::task::spawn_blocking(move || {
        save_track_info(SaveTrackArgs {
            path: &file_path,
            sc_id: &sc_id,
            title: &title,
            artist: &artist,
            artwork_url: artwork_url.as_ref(),
            source_url: source_url.as_ref(),
            artwork_data,
        })
    });

    let (cache_res, meta_res) = tokio::join!(cache_task, meta_task);

    if let Err(err) = cache_res {
        tracing::error!(track_id, "Cache update task panicked: {err:#}");
    }

    match meta_res {
        Err(err) => tracing::error!(track_id, "Metadata task panicked or was cancelled: {err:#}"),
        Ok(Err(err)) => tracing::warn!(track_id, "Failed to save track metadata: {err:#}"),
        _ => {}
    }
}

pub async fn handle_track_failure(
    context: &Context,
    task: &DownloadTask,
    err: anyhow::Error,
    pb: &indicatif::ProgressBar,
) {
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

    pb.println(format!(
        "{} {} — {}",
        "[ERROR]".red().bold(),
        task.display_name(),
        short_err
    ));
}

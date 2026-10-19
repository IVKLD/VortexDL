use std::{collections::HashSet, time::SystemTime};

use colored::Colorize;
use tokio::task::JoinHandle;
use url::Url;

use super::DownloadTask;
use crate::{
    adb,
    database::cache::{CachedMusicTrack, update_cached_tracks_batch},
    downloader::Context,
    storage::{
        LocalMusicTrack,
        metadata::{SaveTrackArgs, save_track_info},
    },
    types::AudioFormat,
    utils::{soundcloud, system_time_to_secs},
};

pub async fn finalize_pipeline_sync(context: &Context, _url: &Url) -> anyhow::Result<()> {
    adb::sync_connected(context.storage.clone(), context.settings.clone()).await;

    soundcloud::update_cached_client_id(&context.client, &context.settings).await;

    Ok(())
}

/// What: Finalizes a downloaded file by writing audio tags and updating the in-memory & on-disk library state.
/// Flow: Called asynchronously immediately after an audio file finishes downloading.
/// Why:  Decouples heavy metadata tagging and artwork embedding from the network download worker,
///       and atomically registers the newly indexed track in `MusicStorage`.
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

    let now = system_time_to_secs(SystemTime::now());

    let mtime = file_metadata
        .and_then(|m| m.modified().ok())
        .map(system_time_to_secs)
        .unwrap_or(now);

    let permalink_url = task.track.permalink_url.clone();
    let metadata = task.track.to_metadata(permalink_url.as_ref());
    let track_id = task.track.id;
    let path_str = task.file_path.to_string_lossy().into_owned();
    let file_path = task.file_path.clone();
    let track_title = task.track.title.clone();
    let track_artist = task.track.artist.clone();
    let artwork_url = task.track.artwork_url.clone();
    let metadata_clone = metadata.clone();

    tokio::task::spawn_blocking(move || {
        let sc_id = track_id.to_string();
        if let Err(err) = save_track_info(SaveTrackArgs {
            path: &file_path,
            sc_id: &sc_id,
            title: &track_title,
            artist: &track_artist,
            artwork_url: artwork_url.as_ref(),
            source_url: permalink_url.as_ref(),
            artwork_data,
        }) {
            tracing::warn!(track_id, "Failed to save track metadata: {err:#}");
        }

        let mut to_update = std::collections::HashMap::new();
        to_update.insert(
            path_str,
            CachedMusicTrack {
                metadata: metadata_clone,
                created_at: now,
                size,
                mtime,
            },
        );
        if let Err(err) = update_cached_tracks_batch(&to_update, &HashSet::new()) {
            tracing::error!(track_id, "Failed to update track cache: {err:#}");
        }
    })
    .await
    .ok();

    context.storage.write().await.update_track(
        task.track.id,
        LocalMusicTrack {
            path: task.file_path.clone(),
            metadata,
            created_at: now,
            size,
            is_archived: false,
        },
    );

    if let Some(ref manager) = context.dm {
        let format = AudioFormat::from_path(&task.file_path);
        manager.update_finished(
            task.track.id,
            format,
            now,
            task.track.permalink_url.as_ref(),
            size,
        );
    }
}

pub async fn handle_track_failure(
    context: &Context,
    task: &DownloadTask,
    err: anyhow::Error,
    pb: &indicatif::ProgressBar,
) {
    let full_err_msg = format!("{:#}", err);
    let mut short_err = full_err_msg;
    if let Some(idx) = short_err.find("url (") {
        short_err.truncate(idx + 4);
        short_err.push_str("...)");
    } else if let Some(idx) = short_err.find("for url") {
        short_err.truncate(idx + 7);
        short_err.push_str(" (...)");
    }

    if let Some(manager) = &context.dm {
        manager.update_failed(task.track.id, &short_err);
    }

    pb.println(format!(
        "{} {} — {}",
        "[ERROR]".red().bold(),
        task.display_name(),
        short_err
    ));
}

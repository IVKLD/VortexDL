use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    time::SystemTime,
};

use anyhow::{Error as AnyhowError, Result};
use colored::Colorize;
use futures::stream::{self, StreamExt};
use indicatif::{MultiProgress, ProgressBar};
use tokio::task::JoinHandle;
use url::Url;

use super::{cancellation, types::{DownloadContext, DownloadTask}};
use crate::{
    devices,
    domain::{AudioFormat, DiscoveredTrack},
    infra::{
        db::{self, cache::{self, CachedTrack}},
        system_time_to_secs,
    },
    library::{
        LibraryTrack,
        metadata::{self, SaveTrackArgs},
    },
    providers::{self, soundcloud},
    terminal,
};

/// End-to-end pipeline that takes a URL (SoundCloud/YouTube) and processes all tracks to completion.
pub async fn run_download_pipeline(ctx: &DownloadContext, url: &Url) -> Result<()> {
    let all_tracks = providers::discover_tracks(ctx, url).await?;
    let to_download = exclude_already_downloaded_tracks(ctx, all_tracks).await;

    if !to_download.is_empty() {
        if let Some(ref m) = ctx.manager {
            for track in &to_download {
                m.add_task(track.clone());
            }
        }
        run_parallel_downloads(ctx, to_download).await;
    } else {
        tracing::info!("Everything synced!");
    }

    finalize_pipeline_sync(ctx, url).await?;
    Ok(())
}

async fn exclude_already_downloaded_tracks(
    ctx: &DownloadContext,
    tracks: Vec<DiscoveredTrack>,
) -> Vec<DiscoveredTrack> {
    let storage = ctx.library.read().await;

    tracks
        .into_iter()
        .filter(|track| {
            let exists = storage.is_track_downloaded(track.id);
            if exists {
                tracing::info!(
                    "Skipping {} - {} (already downloaded)",
                    track.artist,
                    track.title
                );
            }
            !exists
        })
        .collect()
}

pub async fn run_parallel_downloads(ctx: &DownloadContext, tracks: Vec<DiscoveredTrack>) {
    let mp = MultiProgress::new();
    let total_tracks = tracks.len();
    let total_pb = terminal::create_total_progress_bar(&mp, total_tracks as u64);

    let (max_concurrent, naming_template, output_dir) = {
        let s = ctx.settings.read().await;
        (
            s.downloads.max_concurrent as usize,
            s.downloads.naming_template.clone(),
            PathBuf::from(&s.downloads.output_path),
        )
    };

    let results: Vec<bool> = stream::iter(tracks)
        .map(|track| {
            let ctx = ctx.clone();
            let naming_template = naming_template.clone();
            let output_dir = output_dir.clone();
            let pb = total_pb.clone();

            async move { process_track(ctx, track, naming_template, output_dir, pb).await }
        })
        .buffer_unordered(max_concurrent)
        .collect()
        .await;

    total_pb.finish_and_clear();
    let succeeded = results.iter().filter(|&&ok| ok).count();
    let failed = total_tracks.saturating_sub(succeeded);
    tracing::info!("Sync complete. {succeeded} downloaded, {failed} failed or canceled.");
}

async fn process_track(
    ctx: DownloadContext,
    track: DiscoveredTrack,
    naming_template: String,
    output_dir: PathBuf,
    pb: ProgressBar,
) -> bool {
    let mut task = DownloadTask::new(&track, &naming_template, &output_dir);
    let cancel_rx = ctx
        .manager
        .as_ref()
        .and_then(|m| m.get_cancel_receiver(task.track.id));

    if let Some(m) = &ctx.manager {
        m.update_downloading(task.track.id);
    }

    let artwork_handle = providers::spawn_artwork_fetch(&ctx, &task);
    let download_result =
        cancellation::run_with_cancellation(cancel_rx, providers::download_track(&ctx, &mut task))
            .await;

    let success = match download_result {
        Some(Ok(())) => {
            finalize_single_track(ctx, task, artwork_handle, pb.clone()).await;
            true
        }
        Some(Err(err)) => {
            handle_track_failure(&ctx, &task, err, &pb).await;
            false
        }
        None => {
            tracing::info!("Track download canceled: {}", task.display_name());
            if let Some(h) = artwork_handle {
                h.abort();
            }
            if task.file_path.exists() {
                let _ = tokio::fs::remove_file(&task.file_path).await;
            }
            false
        }
    };

    pb.inc(1);
    success
}

async fn finalize_single_track(
    context: DownloadContext,
    task: DownloadTask,
    artwork_handle: Option<JoinHandle<Option<Vec<u8>>>>,
    pb: ProgressBar,
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
        if let Err(err) = metadata::save_track_info(SaveTrackArgs {
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

        let mut to_update = HashMap::new();
        to_update.insert(
            path_str,
            CachedTrack {
                metadata: metadata_clone,
                created_at: now,
                size,
                mtime,
            },
        );
        if let Err(err) = cache::update_cached_tracks_batch(&to_update, &HashSet::new()) {
            tracing::error!(track_id, "Failed to update track cache: {err:#}");
        }
    })
    .await
    .ok();

    context.library.write().await.update_track(
        task.track.id,
        LibraryTrack {
            path: task.file_path.clone(),
            metadata,
            created_at: now,
            size,
            is_archived: false,
        },
    );

    if let Some(ref manager) = context.manager {
        let format = AudioFormat::from_path(&task.file_path);
        manager.update_finished(
            task.track.id,
            format,
            now,
            task.track.permalink_url.as_ref(),
            size,
        );
    }

    let _ = tokio::task::spawn_blocking(move || {
        let _ = db::record_download(size);
    })
    .await;
}

async fn handle_track_failure(
    context: &DownloadContext,
    task: &DownloadTask,
    err: AnyhowError,
    pb: &ProgressBar,
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

    if let Some(manager) = &context.manager {
        manager.update_failed(task.track.id, &short_err);
    }

    pb.println(format!(
        "{} {} — {}",
        "[ERROR]".red().bold(),
        task.display_name(),
        short_err
    ));
}

pub async fn finalize_pipeline_sync(context: &DownloadContext, _url: &Url) -> Result<()> {
    devices::sync_connected(context.library.clone(), context.settings.clone()).await;
    soundcloud::update_cached_client_id(&context.soundcloud, &context.settings).await;
    Ok(())
}

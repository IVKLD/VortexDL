use std::path::PathBuf;

use futures::{
    future::join_all,
    stream::{self, StreamExt},
};
use indicatif::{MultiProgress, ProgressBar};
use tokio::task::JoinHandle;

use super::{
    complete,
    providers::{download_track, spawn_artwork_fetch},
    task::{Context, DownloadTask},
};
use crate::{
    types::DiscoveredMusicTrack, ui::create_total_progress_bar,
    utils::cancellation::run_with_cancellation,
};

pub enum TrackOutcome {
    Success(JoinHandle<()>),
    Failed,
    Canceled,
}

async fn process_parallel_track(
    ctx: Context,
    track: DiscoveredMusicTrack,
    naming_template: String,
    output_dir: PathBuf,
    pb: ProgressBar,
) -> TrackOutcome {
    let mut task = DownloadTask::new(&track, &naming_template, &output_dir);
    let cancel_rx = ctx
        .dm
        .as_ref()
        .and_then(|m| m.get_cancel_receiver(task.track.id));

    if let Some(m) = &ctx.dm {
        m.update_downloading(task.track.id);
    }

    let artwork_handle = spawn_artwork_fetch(&ctx, &task);
    let download_result = run_with_cancellation(cancel_rx, download_track(&ctx, &mut task)).await;

    let outcome = match download_result {
        Some(Ok(())) => TrackOutcome::Success(tokio::spawn(complete::finalize_single_track(
            ctx,
            task,
            artwork_handle,
            pb.clone(),
        ))),
        Some(Err(err)) => {
            complete::handle_track_failure(&ctx, &task, err, &pb).await;
            TrackOutcome::Failed
        }
        None => {
            tracing::info!("Track download canceled: {}", task.display_name());
            if let Some(h) = artwork_handle {
                h.abort();
            }
            if task.file_path.exists() {
                let _ = tokio::fs::remove_file(&task.file_path).await;
            }
            TrackOutcome::Canceled
        }
    };

    pb.inc(1);
    outcome
}

/// What: Concurrently downloads a batch of tracks while honoring cancellation and concurrency limits.
/// Flow: Called by `run_download_pipeline` after track discovery and deduplication.
/// Why:  Uses `stream::buffer_unordered(max_concurrent)` to bound parallel network/disk usage,
///       and spawns independent track processing tasks that monitor cancellation tokens.
pub async fn run_parallel_downloads(ctx: &Context, tracks: Vec<DiscoveredMusicTrack>) {
    let mp = MultiProgress::new();
    let total_tracks = tracks.len();
    let total_pb = create_total_progress_bar(&mp, total_tracks as u64);

    let (max_concurrent, naming_template, output_dir) = {
        let s = ctx.settings.read().await;
        (
            s.downloads.max_concurrent as usize,
            s.downloads.naming_template.clone(),
            PathBuf::from(&s.downloads.output_path),
        )
    };

    let results: Vec<_> = stream::iter(tracks)
        .map(|track| {
            let ctx = ctx.clone();
            let naming_template = naming_template.clone();
            let output_dir = output_dir.clone();
            let pb = total_pb.clone();

            async move {
                tokio::spawn(process_parallel_track(
                    ctx,
                    track,
                    naming_template,
                    output_dir,
                    pb,
                ))
                .await
                .unwrap_or(TrackOutcome::Failed)
            }
        })
        .buffer_unordered(max_concurrent)
        .collect()
        .await;

    let (handles, failed, canceled) = results.into_iter().fold(
        (Vec::new(), 0usize, 0usize),
        |(mut handles, mut failed, mut canceled), outcome| {
            match outcome {
                TrackOutcome::Success(h) => handles.push(h),
                TrackOutcome::Failed => failed += 1,
                TrackOutcome::Canceled => canceled += 1,
            }
            (handles, failed, canceled)
        },
    );
    join_all(handles).await;

    total_pb.finish_and_clear();
    let downloaded = total_tracks.saturating_sub(failed + canceled);
    if canceled > 0 {
        tracing::info!(
            "Sync complete. {downloaded} downloaded, {failed} failed, {canceled} canceled."
        );
    } else {
        tracing::info!("Sync complete. {downloaded} downloaded, {failed} failed.");
    }
}

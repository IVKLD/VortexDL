mod complete;
mod filter;
pub mod providers;
mod task;

use std::{collections::HashSet, path::PathBuf};

use anyhow::Result;
use futures::{
    future::join_all,
    stream::{self, StreamExt},
};
use indicatif::MultiProgress;
pub use providers::{soundcloud, youtube};
pub use task::{Context, DownloadTask};
use url::Url;

use crate::{
    types::DiscoveredMusicTrack, ui::create_total_progress_bar,
    utils::cancellation::run_with_cancellation,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourcePlatform {
    SoundCloud,
    YouTube,
}

impl SourcePlatform {
    pub fn detect(url: &Url) -> Self {
        if yt_audio_downloader::is_youtube_url(url.as_str()) {
            Self::YouTube
        } else {
            Self::SoundCloud
        }
    }

    pub fn from_track(track: &DiscoveredMusicTrack) -> Self {
        track
            .permalink_url
            .as_ref()
            .map(Self::detect)
            .unwrap_or(Self::SoundCloud)
    }

    pub fn spawn_artwork_fetch(
        &self,
        ctx: &Context,
        track: &DiscoveredMusicTrack,
    ) -> Option<tokio::task::JoinHandle<Option<Vec<u8>>>> {
        soundcloud::spawn_artwork_fetch(ctx, track.artwork_url.as_ref())
    }

    pub async fn download_track(&self, ctx: &Context, task: &mut DownloadTask) -> Result<()> {
        match self {
            Self::YouTube => {
                let url_or_id = task
                    .track
                    .permalink_url
                    .as_ref()
                    .map(|u| u.as_str())
                    .unwrap_or("");
                youtube::download_youtube_track(ctx, task, url_or_id).await
            }
            Self::SoundCloud => {
                let stream = soundcloud::resolve_stream_source(ctx, task.track.id).await?;
                if task.track.permalink_url.is_none() {
                    task.track.permalink_url = Url::parse(stream.url()).ok();
                }
                soundcloud::download_soundcloud_track(ctx, task, stream).await
            }
        }
    }
}

/// Resolves music URLs (SoundCloud or YouTube), checks database to avoid re-downloading, orchestrates parallel execution, and runs post-sync tasks.
pub async fn run_download_pipeline(ctx: &Context, url: &Url) -> Result<()> {
    let platform = SourcePlatform::detect(url);

    let all_tracks = match platform {
        SourcePlatform::YouTube => youtube::discover_youtube_tracks(url).await?,
        SourcePlatform::SoundCloud => soundcloud::resolve_tracks_from_url(ctx, url).await?,
    };

    let remote_ids: HashSet<i64> = all_tracks.iter().map(|track| track.id).collect();
    let to_download = filter::exclude_already_downloaded_tracks(ctx, all_tracks).await;

    if !to_download.is_empty() {
        if let Some(ref m) = ctx.dm {
            for track in &to_download {
                m.add_task(track.clone());
            }
        }
        run_parallel_downloads(ctx, to_download).await;
    } else {
        tracing::info!("Everything synced!");
    }

    complete::finalize_pipeline_sync(ctx, url, &remote_ids).await?;

    Ok(())
}

async fn run_parallel_downloads(ctx: &Context, tracks: Vec<DiscoveredMusicTrack>) {
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
            let pb = total_pb.clone();
            let output_dir = output_dir.clone();
            let naming_template = naming_template.clone();

            async move {
                let mut task = DownloadTask::new(&track, &naming_template, &output_dir);
                let cancel_rx = ctx
                    .dm
                    .as_ref()
                    .and_then(|m| m.get_cancel_receiver(task.track.id));

                if let Some(m) = &ctx.dm {
                    m.update_downloading(task.track.id);
                }

                let platform = SourcePlatform::from_track(&task.track);
                let artwork_handle = platform.spawn_artwork_fetch(&ctx, &task.track);

                let download_result =
                    run_with_cancellation(cancel_rx, platform.download_track(&ctx, &mut task))
                        .await;

                let handle = match download_result {
                    Some(Ok(())) => Some(tokio::spawn(complete::finalize_single_track(
                        ctx,
                        task,
                        artwork_handle,
                        pb.clone(),
                    ))),
                    Some(Err(err)) => {
                        complete::handle_track_failure(&ctx, &task, err, &pb).await;
                        None
                    }
                    None => {
                        tracing::info!("Track download canceled: {}", task.display_name());
                        if let Some(h) = artwork_handle {
                            h.abort();
                        }
                        if task.file_path.exists() {
                            let _ = tokio::fs::remove_file(&task.file_path).await;
                        }
                        None
                    }
                };

                pb.inc(1);
                handle
            }
        })
        .buffer_unordered(max_concurrent)
        .collect()
        .await;

    let failed = results.iter().filter(|r| r.is_none()).count();
    let handles: Vec<_> = results.into_iter().flatten().collect();
    join_all(handles).await;

    total_pb.finish_and_clear();
    tracing::info!(
        "Sync complete. {} downloaded, {} failed.",
        total_tracks - failed,
        failed
    );
}

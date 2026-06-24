mod complete;
pub(crate) mod discovery;
mod download;
mod filter_existing;
mod resolve;

use std::{collections::HashSet, path::PathBuf};

use anyhow::Result;
use futures::{
    future::join_all,
    stream::{self, StreamExt},
};
use indicatif::MultiProgress;
use tokio::task::JoinHandle;

pub use crate::types::core::Context;
use crate::{
    api::download_manager::NewTask,
    types::core::DiscoveredMusicTrack,
    ui::{create_spinner, create_total_progress_bar},
    utils::filename::clean_filename,
};

#[derive(Clone)]
pub(crate) struct DownloadTask {
    pub id: i64,
    pub title: String,
    pub artist: String,
    pub artwork_url: Option<String>,
    pub position: Option<u32>,
    pub pb: indicatif::ProgressBar,
    pub file_path: PathBuf,
}

impl DownloadTask {
    pub fn display_name(&self) -> &str {
        self.file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&self.title)
    }
}

/// Resolves SoundCloud URLs, checks database to avoid re-downloading, orchestrates parallel execution, and runs post-sync tasks.
pub async fn run_download_pipeline(ctx: &Context, url: &str) -> Result<()> {
    let all_tracks = resolve::resolve_tracks_from_url(ctx, url).await?;

    let remote_ids: HashSet<i64> = all_tracks.iter().map(|track| track.id).collect();
    let to_download = filter_existing::exclude_already_downloaded_tracks(ctx, all_tracks).await;

    if !to_download.is_empty() {
        if let Some(ref m) = ctx.dm {
            for track in &to_download {
                m.add_task(NewTask {
                    id: track.id,
                    title: track.title.clone(),
                    artist: track.artist.clone(),
                    artwork_url: track.artwork_url.clone(),
                    position: track.position,
                });
            }
        }
        run_parallel_downloads(ctx, to_download).await;
    } else {
        tracing::info!("Everything synced!");
    }

    complete::finalize_pipeline_sync(ctx, url, &remote_ids).await?;

    Ok(())
}

/// Runs resolution, downloading and tagging steps sequentially for a single track.
async fn run_track_download_pipeline(
    context: Context,
    task: DownloadTask,
) -> Option<JoinHandle<()>> {
    let pipeline = async {
        if let Some(manager) = &context.dm {
            manager.update_downloading(task.id);
        }
        let display_name = task.display_name().to_string();
        task.pb.set_message(format!("Downloading: {display_name}"));

        let artwork_handle = resolve::spawn_artwork_fetch(&context, task.artwork_url.as_deref());

        let proto = resolve::resolve_stream_source(&context, task.id).await?;

        let url = proto.url().to_string();

        download::download_single_track(&context, &task, proto).await?;

        Ok((artwork_handle, url))
    };

    match pipeline.await {
        Ok((artwork_handle, url)) => Some(tokio::spawn(complete::finalize_single_track(
            context,
            task,
            artwork_handle,
            url,
        ))),
        Err(err) => {
            complete::handle_track_failure(&context, &task, err).await;
            None
        }
    }
}

async fn run_parallel_downloads(ctx: &Context, tracks: Vec<DiscoveredMusicTrack>) {
    let mp = MultiProgress::new();
    let total_tracks = tracks.len();
    let total_pb = create_total_progress_bar(&mp, total_tracks as u64);

    let (max_concurrent, naming_template) = {
        let s = ctx.settings.read().await;
        (
            s.downloads.max_concurrent as usize,
            s.downloads.naming_template.clone(),
        )
    };
    let output_dir = PathBuf::from(&ctx.storage.read().await.base_path);

    let results: Vec<_> = stream::iter(tracks)
        .map(|mut track| {
            track.artwork_url = track.artwork_url.map(|url| {
                if url.contains("-large") {
                    url.replacen("-large", "-t1080x1080", 1)
                } else {
                    url
                }
            });

            let ctx = ctx.clone();
            let mp = mp.clone();
            let total_pb = total_pb.clone();
            let filename = format!(
                "{}.mp3",
                clean_filename(
                    &naming_template
                        .replace("{artist}", &track.artist)
                        .replace("{title}", &track.title),
                )
            );
            let file_path = output_dir.join(filename);

            async move {
                let pb = create_spinner(&mp);

                let task = DownloadTask {
                    id: track.id,
                    title: track.title,
                    artist: track.artist,
                    artwork_url: track.artwork_url,
                    position: track.position,
                    pb: pb.clone(),
                    file_path,
                };

                let result = run_track_download_pipeline(ctx, task).await;
                total_pb.inc(1);
                pb.finish_and_clear();
                result
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

mod complete;
pub(crate) mod discovery;
mod download;
mod filter_existing;
mod resolve;

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Result;
use futures::{
    future::join_all,
    stream::{self, StreamExt},
};
use indicatif::MultiProgress;
use tokio::{sync::RwLock, task::JoinHandle};
use url::Url;

use crate::{
    api::{download_manager::DownloadManager, state::AppState},
    settings::SettingsManager,
    storage::MusicStorage,
    types::DiscoveredMusicTrack,
    ui::create_total_progress_bar,
    utils::filename::clean_filename,
};

#[derive(Clone)]
pub struct Context {
    pub storage: Arc<RwLock<MusicStorage>>,
    pub client: Arc<soundcloud_rs::Client>,
    pub dm: Option<Arc<DownloadManager>>,
    pub settings: SettingsManager,
}

impl Context {
    pub fn from_state(state: &AppState) -> Self {
        Self {
            storage: state.storage.clone(),
            client: state.client.clone(),
            dm: None,
            settings: state.settings.clone(),
        }
    }

    pub fn with_dm(mut self, dm: Arc<DownloadManager>) -> Self {
        self.dm = Some(dm);
        self
    }
}

#[derive(Clone)]
pub(crate) struct DownloadTask {
    pub track: DiscoveredMusicTrack,
    pub file_path: PathBuf,
}

impl DownloadTask {
    pub fn display_name(&self) -> &str {
        self.file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&self.track.title)
    }

    pub fn new(
        track: &DiscoveredMusicTrack,
        naming_template: &str,
        output_dir: &Path,
    ) -> Self {
        let formatted = naming_template
            .replace("{artist}", &track.artist)
            .replace("{title}", &track.title);
        let filename = format!("{}.mp3", clean_filename(&formatted));

        Self {
            track: track.clone(),
            file_path: output_dir.join(filename),
        }
    }
}

/// Resolves SoundCloud URLs, checks database to avoid re-downloading, orchestrates parallel execution, and runs post-sync tasks.
pub async fn run_download_pipeline(ctx: &Context, url: &Url) -> Result<()> {
    let all_tracks = resolve::resolve_tracks_from_url(ctx, url).await?;

    let remote_ids: HashSet<i64> = all_tracks.iter().map(|track| track.id).collect();
    let to_download = filter_existing::exclude_already_downloaded_tracks(ctx, all_tracks).await;

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

/// Runs resolution, downloading and tagging steps sequentially for a single track.
async fn run_track_download_pipeline(
    context: Context,
    mut task: DownloadTask,
    pb: indicatif::ProgressBar,
) -> Option<JoinHandle<()>> {
    if let Some(manager) = &context.dm {
        manager.update_downloading(task.track.id);
    }

    let artwork_handle = resolve::spawn_artwork_fetch(&context, task.track.artwork_url.as_ref());

    let stream = match resolve::resolve_stream_source(&context, task.track.id).await {
        Ok(s) => s,
        Err(err) => {
            complete::handle_track_failure(&context, &task, err, &pb).await;
            return None;
        }
    };

    if task.track.permalink_url.is_none() {
        task.track.permalink_url = Url::parse(stream.url()).ok();
    }

    if let Err(err) = download::download_single_track(&context, &task, stream).await {
        complete::handle_track_failure(&context, &task, err, &pb).await;
        return None;
    }

    Some(tokio::spawn(complete::finalize_single_track(
        context,
        task,
        artwork_handle,
        pb,
    )))
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
            let total_pb = total_pb.clone();
            let output_dir = output_dir.clone();
            let naming_template = naming_template.clone();

            async move {
                let task = DownloadTask::new(&track, &naming_template, &output_dir);

                let pb_clone = total_pb.clone();
                let result = run_track_download_pipeline(ctx, task, pb_clone).await;
                total_pb.inc(1);
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

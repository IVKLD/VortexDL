pub mod pipeline;

use std::path::PathBuf;

use colored::Colorize;
use futures::{
    future::join_all,
    stream::{self, StreamExt},
};
use indicatif::MultiProgress;

use crate::{
    downloader::{Context, TrackDownload, core::pipeline as pl},
    ui::{create_spinner, create_total_progress_bar},
};

pub async fn run_download_batch(ctx: &Context, tracks: Vec<TrackDownload>) {
    let mp = MultiProgress::new();
    let total_tracks = tracks.len();
    let total_pb = create_total_progress_bar(&mp, total_tracks as u64);

    let max_concurrent = ctx.settings.read().await.downloads.max_concurrent as usize;
    let output_dir = PathBuf::from(&ctx.storage.read().await.base_path);

    let results: Vec<_> = stream::iter(tracks)
        .map(|track| {
            let ctx = ctx.clone();
            let mp = mp.clone();
            let total_pb = total_pb.clone();
            let output_dir = output_dir.clone();

            async move {
                let pb = create_spinner(&mp);
                let file_path = track.path(&output_dir);

                let task = pl::DownloadTask {
                    id: track.id,
                    title: track.title,
                    artist: track.artist,
                    artwork_url: track.artwork_url,
                    position: track.position,
                    pb: pb.clone(),
                    file_path,
                };

                let result = pl::run_track_pipeline(ctx, task).await;
                total_pb.inc(1);
                pb.finish_and_clear();
                result
            }
        })
        .buffer_unordered(max_concurrent)
        .collect()
        .await;

    let failed = results.iter().filter(|r| r.is_none()).count();
    join_all(results.into_iter().flatten()).await;

    total_pb.finish_and_clear();
    println!(
        "{} Sync complete. {} downloaded, {} failed.",
        "[SUCCESS]".green().bold(),
        total_tracks - failed,
        failed
    );
}

pub mod pipeline;

use colored::Colorize;
use futures::{
    future::join_all,
    stream::{self, StreamExt},
};
use indicatif::MultiProgress;

use std::path::PathBuf;

use crate::{
    downloader::{Context, TrackDownload, core::pipeline as pl},
    ui::{create_spinner, create_total_progress_bar},
    utils::filename::clean_filename,
};

pub async fn run_download_batch(ctx: &Context, tracks: Vec<TrackDownload>) {
    let mp = MultiProgress::new();
    let total_tracks = tracks.len();
    let total_pb = create_total_progress_bar(&mp, total_tracks as u64);

    let (max_concurrent, output_path) = {
        let s = ctx.settings.read().await;
        (
            s.downloads.max_concurrent as usize,
            s.downloads.output_path.clone(),
        )
    };

    let results: Vec<_> = stream::iter(tracks)
        .map(|track| {
            let ctx = ctx.clone();
            let mp = mp.clone();
            let total_pb = total_pb.clone();
            let output_dir = PathBuf::from(&output_path);

            async move {
                let pb = create_spinner(&mp);
                let display_name = clean_filename(&format!("{} - {}", track.artist, track.title));
                let file_path = output_dir.join(format!("{display_name}.mp3"));

                let task = pl::DownloadTask {
                    id: track.id,
                    title: track.title,
                    artist: track.artist,
                    display_name,
                    artwork_url: track.artwork_url,
                    position: track.position,
                    pb: pb.clone(),
                    output_dir,
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

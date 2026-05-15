use std::{sync::Arc, time::Duration};

use colored::Colorize;
use futures::{
    future::join_all,
    stream::{self, StreamExt},
};
use indicatif::MultiProgress;
use reqwest::Client as HttpClient;

use crate::{
    downloader::{Context, TrackDownload, core::pipeline},
    ui::{create_spinner, create_total_progress_bar},
    utils::filename::format_track_filename,
};

/// Executes the download pipeline for multiple tracks in parallel.
pub(crate) async fn run_download_batch(ctx: &Context, tracks: Vec<TrackDownload>) {
    let mp = MultiProgress::new();
    let total_tracks = tracks.len();
    let total_pb = create_total_progress_bar(&mp, total_tracks as u64);
    let http = Arc::new(
        HttpClient::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default(),
    );

    let (max_concurrent, output_path) = {
        let s = ctx.settings.read().await;
        (
            s.downloads.max_concurrent as usize,
            s.downloads.output_path.clone(),
        )
    };

    let results: Vec<_> = stream::iter(tracks)
        .map(|track| {
            let mut ctx = ctx.clone();
            ctx.http = Arc::clone(&http);
            let mp = mp.clone();
            let total_pb = total_pb.clone();
            let output_dir = output_path.clone();

            async move {
                let pb = create_spinner(&mp);
                let filename = format_track_filename(&track.artist, &track.title);
                let file_path = format!("{}/{}.mp3", output_dir, filename);

                let task = pipeline::DownloadTask {
                    id: track.id,
                    title: track.title.clone(),
                    artist: track.artist.clone(),
                    artwork_url: track.artwork_url.clone(),
                    position: track.position,
                    pb: pb.clone(),
                    output_dir,
                    file_path,
                };

                let result = pipeline::run_track_pipeline(ctx, task).await;
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

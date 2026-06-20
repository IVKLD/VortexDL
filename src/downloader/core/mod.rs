pub mod pipeline;

use std::path::PathBuf;

use colored::Colorize;
use futures::{
    future::join_all,
    stream::{self, StreamExt},
};
use indicatif::MultiProgress;

use crate::{
    downloader::{Context, DiscoveredTrack, core::pipeline as pl},
    ui::{create_spinner, create_total_progress_bar},
    utils::filename::clean_filename,
};

pub async fn run_download_batch(ctx: &Context, tracks: Vec<DiscoveredTrack>) {
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
            if let Some(ref url) = track.artwork_url {
                track.artwork_url = Some(url.replacen("-large", "-t1080x1080", 1));
            }

            let ctx = ctx.clone();
            let mp = mp.clone();
            let total_pb = total_pb.clone();
            let filename = naming_template
                .replace("{artist}", &track.artist)
                .replace("{title}", &track.title);
            let clean_name = clean_filename(&filename);
            let file_path = output_dir.join(clean_name).with_extension("mp3");

            async move {
                let pb = create_spinner(&mp);

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
    let handles: Vec<_> = results.into_iter().flatten().collect();
    join_all(handles).await;

    total_pb.finish_and_clear();
    println!(
        "{} Sync complete. {} downloaded, {} failed.",
        "[SUCCESS]".green().bold(),
        total_tracks - failed,
        failed
    );
}

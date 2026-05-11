use std::{sync::Arc, time::Duration};

use colored::Colorize;
use futures::{
    future::join_all,
    stream::{self, StreamExt},
};
use indicatif::MultiProgress;
use reqwest::Client as HttpClient;

use crate::{
    downloader::{Context, utils::clean_filename},
    ui::{create_spinner, create_total_progress_bar},
};

pub(in crate::downloader) mod artwork;
pub(in crate::downloader) mod hls;
pub(in crate::downloader) mod progressive;
pub(in crate::downloader) mod track;
pub(in crate::downloader) mod verification;

#[derive(Clone, Debug)]
pub(in crate::downloader) struct TrackDownload {
    pub id: i64,
    pub filename: String,
    pub artwork_url: Option<String>,
}

pub(in crate::downloader) async fn run_parallel_download(
    ctx: &Context,
    tracks: Vec<TrackDownload>,
) {
    let mp = MultiProgress::new();
    let total_pb = create_total_progress_bar(&mp, tracks.len() as u64);

    let http = HttpClient::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap_or_default();

    let max_concurrent = {
        let s = ctx.settings.read().await;
        s.downloads.max_concurrent as usize
    };

    let results: Vec<_> = stream::iter(tracks)
        .map(|track_info| {
            let (client, http, storage, dm, mp, total_pb, settings) = (
                Arc::clone(&ctx.client),
                http.clone(),
                Arc::clone(&ctx.storage),
                ctx.dm.clone(),
                mp.clone(),
                total_pb.clone(),
                Arc::clone(&ctx.settings),
            );

            async move {
                let pb = create_spinner(&mp);

                let filename = clean_filename(&track_info.filename);

                let output_dir = {
                    let s = settings.read().await;
                    s.downloads.output_path.clone()
                };

                let file_path = format!("{}/{}.mp3", output_dir, filename);

                let ctx = track::Context {
                    client: &client,
                    http: &http,
                    storage: &storage,
                    dm: dm.as_ref(),
                    settings: &settings,
                };

                let task = track::Task {
                    id: track_info.id,
                    filename,
                    artwork_url: track_info.artwork_url.as_deref(),
                    pb: &pb,
                    output_dir,
                    file_path,
                };

                let result = track::initiate_track_download(&ctx, &task).await;
                total_pb.inc(1);
                pb.finish_and_clear();
                result
            }
        })
        .buffer_unordered(max_concurrent)
        .collect()
        .await;

    let failed = results.iter().filter(|r| r.is_none()).count();
    let downloaded = results.len() - failed;

    join_all(results.into_iter().flatten()).await;
    total_pb.finish_and_clear();

    println!(
        "{} Sync complete. {} downloaded, {} failed.",
        colored::Colorize::green("[SUCCESS]").bold(),
        downloaded,
        failed
    );
}

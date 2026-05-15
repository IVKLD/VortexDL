use std::time::Duration;

use anyhow::{Result, anyhow};
use futures::StreamExt;
use soundcloud_rs::{Identifier, StreamType, Track};
use tokio::{fs, io::AsyncWriteExt, time::sleep};

use crate::{
    downloader::{
        Context,
        core::pipeline::{DownloadTask, resolve::DownloadProtocol},
    },
    ui,
    utils::{filename::format_track_filename, verification::verify_file},
};

pub async fn download_with_retries(
    ctx: &Context,
    task: &DownloadTask,
    track: &Track,
    sc_id: &Identifier,
    protocol: DownloadProtocol,
) -> Result<()> {
    let max_retries = ctx.settings.read().await.max_retries;
    let mut attempts_left = max_retries.max(1);

    loop {
        let result = match &protocol {
            DownloadProtocol::Progressive(url) => try_download_progressive(ctx, task, url).await,
            DownloadProtocol::Hls(_) => try_download_hls(ctx, task, track, sc_id).await,
        };

        let Err(e) = result else {
            return Ok(());
        };

        attempts_left -= 1;

        if attempts_left == 0 {
            tokio::fs::remove_file(&task.file_path).await.ok();
            return Err(e);
        }

        task.pb.set_message(format!(
            "Retrying ({} left): {}",
            attempts_left,
            format_track_filename(&task.artist, &task.title)
        ));
        sleep(Duration::from_secs(1)).await;
    }
}

async fn try_download_progressive(ctx: &Context, task: &DownloadTask, url: &str) -> Result<()> {
    let response = ctx.http.get(url).send().await?.error_for_status()?;
    let total = response.content_length().unwrap_or(0);

    task.pb.set_message(format!(
        "Downloading Music & Art: {}",
        format_track_filename(&task.artist, &task.title)
    ));

    ui::upgrade_to_download_bar(&task.pb, total);

    let mut file = fs::File::create(&task.file_path).await?;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| anyhow!("Stream error: {}", e))?;
        file.write_all(&chunk).await?;

        task.pb.inc(chunk.len() as u64);

        if let Some(m) = &ctx.dm {
            m.update_progress(task.id, task.pb.position(), total).await;
        }
    }

    drop(file);
    verify_file(&task.file_path, total).await?;

    Ok(())
}

async fn try_download_hls(
    ctx: &Context,
    task: &DownloadTask,
    track: &Track,
    sc_id: &Identifier,
) -> Result<()> {
    let filename = format_track_filename(&task.artist, &task.title);

    task.pb
        .set_message(format!("Downloading Music & Art (HLS): {}", filename));

    ctx.client
        .download_track(
            track,
            sc_id,
            Some(&StreamType::Hls),
            Some(&task.output_dir),
            Some(&filename),
        )
        .await
        .map_err(|e| anyhow!("HLS download failed: {}", e))?;

    verify_file(&task.file_path, 0).await
}

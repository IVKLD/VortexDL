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
    utils::{soundcloud::init_client_with_settings, verification::verify_file},
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
            DownloadProtocol::Progressive { url, proxy_url } => {
                try_download_progressive(ctx, task, url, proxy_url.as_deref()).await
            }
            DownloadProtocol::Hls { proxy_url, .. } => {
                try_download_hls(ctx, task, track, sc_id, proxy_url.as_deref()).await
            }
        };

        if result.is_ok() {
            return Ok(());
        }

        attempts_left -= 1;
        if attempts_left == 0 {
            fs::remove_file(&task.file_path).await.ok();
            return result;
        }

        task.pb.set_message(format!(
            "Retrying ({attempts_left} left): {}",
            task.display_name
        ));
        sleep(Duration::from_secs(1)).await;
    }
}

async fn try_download_progressive(
    ctx: &Context,
    task: &DownloadTask,
    url: &str,
    proxy_url: Option<&str>,
) -> Result<()> {
    let client = match proxy_url {
        Some(proxy) => reqwest::Client::builder()
            .proxy(reqwest::Proxy::all(proxy)?)
            .build()?,
        None => ctx.http.clone(),
    };
    let response = client.get(url).send().await?.error_for_status()?;
    let total = response.content_length().unwrap_or(0);

    task.pb
        .set_message(format!("Downloading: {}", task.display_name));
    ui::upgrade_to_download_bar(&task.pb, total);

    let mut file = fs::File::create(&task.file_path).await?;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| anyhow!("Stream error: {e}"))?;
        file.write_all(&chunk).await?;
        task.pb.inc(chunk.len() as u64);

        if let Some(m) = &ctx.dm {
            m.update_progress(task.id, task.pb.position(), total);
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
    proxy_url: Option<&str>,
) -> Result<()> {
    task.pb
        .set_message(format!("Downloading (HLS): {}", task.display_name));

    if let Some(proxy) = proxy_url {
        let client = init_client_with_settings(&*ctx.settings.read().await, Some(proxy))
            .await
            .map_err(|e| anyhow!("Failed to build proxied client: {e}"))?;
        client
            .download_track(
                track,
                sc_id,
                Some(&StreamType::Hls),
                task.output_dir.to_str(),
                Some(&task.display_name),
            )
            .await
    } else {
        ctx.client
            .download_track(
                track,
                sc_id,
                Some(&StreamType::Hls),
                task.output_dir.to_str(),
                Some(&task.display_name),
            )
            .await
    }
    .map_err(|e| anyhow!("HLS download failed: {e}"))?;

    verify_file(&task.file_path, 0).await
}

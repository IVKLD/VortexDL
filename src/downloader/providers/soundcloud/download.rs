use std::time::Duration;

use anyhow::{Result, anyhow};
use futures::StreamExt;
use tokio::{fs, io::AsyncWriteExt, time::sleep};

use super::resolve::StreamSource;
use crate::{
    downloader::{Context, DownloadTask},
    utils::{http::build_http_client, soundcloud, verification::verify},
};

pub async fn download_soundcloud_track(
    context: &Context,
    task: &DownloadTask,
    stream_source: StreamSource,
) -> Result<()> {
    let max_retries = context.settings.read().await.system.max_retries;
    let mut attempts_left = max_retries.max(1);

    loop {
        let result = match &stream_source {
            StreamSource::Progressive { url, proxy_url } => {
                download_progressive(context, task, url, proxy_url.as_deref()).await
            }
            StreamSource::Hls { url, proxy_url } => {
                download_hls(context, task, url, proxy_url.as_deref()).await
            }
        };

        match result {
            Ok(()) => return Ok(()),
            Err(err) => {
                attempts_left -= 1;
                if attempts_left == 0 {
                    return Err(err);
                }
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

async fn download_progressive(
    context: &Context,
    task: &DownloadTask,
    url: &str,
    proxy_url: Option<&str>,
) -> Result<()> {
    let settings = context.settings.read().await;
    let active_proxy = proxy_url.or_else(|| settings.network.get_proxy_url());
    let client = build_http_client(active_proxy, 5, 30);
    let response = client.get(url).send().await?.error_for_status()?;
    let total = response
        .content_length()
        .unwrap_or_else(|| match task.track.duration_ms {
            Some(ms) => ((ms as f64 / 1000.0).max(1.0) * 16_000.0) as u64,
            None => 2_500_000,
        });

    let mut file = fs::File::create(&task.file_path).await?;
    let mut stream = response.bytes_stream();
    let mut position = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|err| anyhow!("Stream error: {err}"))?;
        file.write_all(&chunk).await?;
        position += chunk.len() as u64;

        if let Some(manager) = &context.dm {
            manager.update_progress(task.track.id, position, total);
        }
    }

    drop(file);
    verify(&task.file_path, total).await?;
    Ok(())
}

async fn download_hls(
    context: &Context,
    task: &DownloadTask,
    stream_url: &str,
    proxy_url: Option<&str>,
) -> Result<()> {
    let settings = context.settings.read().await;
    let active_proxy = proxy_url.or_else(|| settings.network.get_proxy_url());

    let proxied_client;
    let client: &soundcloud_rs::Client = if let Some(proxy) = active_proxy {
        proxied_client = soundcloud::ClientBuilder::new(&settings)
            .with_proxy(Some(proxy))
            .build()
            .await
            .map_err(|err| anyhow!("Failed to build proxied client: {err}"))?;
        &proxied_client
    } else {
        &context.client
    };

    let track_id = task.track.id;
    let path_clone = task.file_path.clone();
    let dm = context.dm.clone();
    let duration_ms = task.track.duration_ms;

    let progress_task = tokio::spawn(async move {
        let mut has_existed = false;
        let mut missing_count = 0;
        loop {
            sleep(Duration::from_millis(500)).await;
            match fs::metadata(&path_clone).await {
                Ok(meta) => {
                    has_existed = true;
                    missing_count = 0;
                    if let Some(manager) = &dm {
                        let estimated_total = match duration_ms {
                            Some(ms) => ((ms as f64 / 1000.0).max(1.0) * 24_000.0) as u64,
                            None => 3_500_000,
                        };
                        manager.update_progress(track_id, meta.len(), estimated_total);
                    }
                }
                Err(_) => {
                    if has_existed {
                        break;
                    }
                    missing_count += 1;
                    if missing_count > 20 {
                        break;
                    }
                }
            }
        }
    });

    let download_res = client
        .download_hls_to_file(stream_url, &task.file_path)
        .await;
    progress_task.abort();

    download_res.map_err(|err| anyhow!("HLS download failed: {err}"))?;

    verify(&task.file_path, 0).await?;
    Ok(())
}

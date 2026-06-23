use std::time::Duration;

use anyhow::{Result, anyhow};
use futures::StreamExt;
use tokio::{fs, io::AsyncWriteExt, time::sleep};

use crate::{
    downloader::Context,
    ui,
    utils::{soundcloud::init_client_with_settings, verification::verify},
};
use super::super::DownloadTask;
use super::resolve::StreamSource;

pub async fn download_single_track(
    context: &Context,
    task: &DownloadTask,
    stream_source: StreamSource,
) -> Result<()> {
    let max_retries = context.settings.read().await.max_retries;
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
                fs::remove_file(&task.file_path).await.ok();
                if attempts_left == 0 {
                    return Err(err);
                }

                task.pb.set_message(format!(
                    "Retrying ({attempts_left} left): {}",
                    task.display_name()
                ));
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
    let client = match proxy_url {
        Some(proxy) => reqwest::Client::builder()
            .proxy(reqwest::Proxy::all(proxy)?)
            .build()?,
        None => context.http.clone(),
    };
    let response = client.get(url).send().await?.error_for_status()?;
    let total = response.content_length().unwrap_or(0);

    task.pb
        .set_message(format!("Downloading: {}", task.display_name()));
    ui::upgrade_to_download_bar(&task.pb, total);

    let mut file = fs::File::create(&task.file_path).await?;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|err| anyhow!("Stream error: {err}"))?;
        file.write_all(&chunk).await?;
        task.pb.inc(chunk.len() as u64);

        if let Some(manager) = &context.dm {
            manager.update_progress(task.id, task.pb.position(), total);
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
    task.pb
        .set_message(format!("Downloading (HLS): {}", task.display_name()));

    let proxied_client;
    let client = if let Some(proxy) = proxy_url {
        proxied_client = init_client_with_settings(&*context.settings.read().await, Some(proxy))
            .await
            .map_err(|err| anyhow!("Failed to build proxied client: {err}"))?;
        &proxied_client
    } else {
        &context.client
    };

    client
        .download_hls_to_file(stream_url, &task.file_path)
        .await
        .map_err(|err| anyhow!("HLS download failed: {err}"))?;

    verify(&task.file_path, 0).await
}

use std::{path::PathBuf, time::Duration};

use anyhow::{Result, anyhow};
use futures::StreamExt;
use tokio::{fs, io::AsyncWriteExt, time::sleep};

use super::{DownloadTask, resolve::StreamSource};
use crate::{
    downloader::Context,
    utils::{http::build_http_client, soundcloud::SoundCloudClientBuilder, verification::verify},
};

pub async fn download_single_track(
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

struct TempFileGuard {
    path: PathBuf,
    disarmed: bool,
}

impl TempFileGuard {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            disarmed: false,
        }
    }

    fn disarm(&mut self) {
        self.disarmed = true;
    }
}

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        if !self.disarmed {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

struct AbortOnDrop(tokio::task::JoinHandle<()>);

impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        self.0.abort();
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
        .unwrap_or_else(|| task.track.estimated_bytes());

    let tmp_path = task.file_path.with_extension("mp3.tmp");
    let mut temp_guard = TempFileGuard::new(tmp_path.clone());

    let mut file = fs::File::create(&tmp_path).await?;
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
    verify(&tmp_path, total).await?;
    fs::rename(&tmp_path, &task.file_path).await?;
    temp_guard.disarm();
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
        proxied_client = SoundCloudClientBuilder::new(&settings)
            .with_proxy(Some(proxy))
            .build()
            .await
            .map_err(|err| anyhow!("Failed to build proxied client: {err}"))?;
        &proxied_client
    } else {
        &context.client
    };

    let tmp_path = task.file_path.with_extension("mp3.tmp");
    let mut temp_guard = TempFileGuard::new(tmp_path.clone());
    let estimated_total = task.track.estimated_bytes();

    let _monitor_guard = context.dm.clone().map(|dm| {
        let tmp_path = tmp_path.clone();
        let track_id = task.track.id;
        AbortOnDrop(tokio::spawn(async move {
            loop {
                sleep(Duration::from_millis(200)).await;
                if let Ok(meta) = fs::metadata(&tmp_path).await
                    && meta.len() > 0
                {
                    dm.update_progress(track_id, meta.len(), estimated_total);
                }
            }
        }))
    });

    client
        .download_hls_to_file(stream_url, &tmp_path)
        .await
        .map_err(|err| anyhow!("HLS download failed: {err}"))?;

    verify(&tmp_path, 0).await?;
    fs::rename(&tmp_path, &task.file_path).await?;
    temp_guard.disarm();
    Ok(())
}

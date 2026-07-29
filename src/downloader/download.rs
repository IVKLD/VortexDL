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
        .unwrap_or_else(|| {
            match task.track.duration_ms {
                Some(ms) => {
                    let secs = (ms as f64 / 1000.0).max(1.0);
                    (secs * 16_000.0) as u64
                }
                None => 2_500_000,
            }
        });

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

    let track_id = task.track.id;
    let tmp_path_clone = tmp_path.clone();
    let dm = context.dm.clone();
    let duration_ms = task.track.duration_ms;
    tokio::spawn(async move {
        let mut has_existed = false;
        let mut missing_count = 0;
        loop {
            sleep(Duration::from_millis(500)).await;
            match fs::metadata(&tmp_path_clone).await {
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

    client
        .download_hls_to_file(stream_url, &tmp_path)
        .await
        .map_err(|err| anyhow!("HLS download failed: {err}"))?;

    verify(&tmp_path, 0).await?;
    fs::rename(&tmp_path, &task.file_path).await?;
    temp_guard.disarm();
    Ok(())
}

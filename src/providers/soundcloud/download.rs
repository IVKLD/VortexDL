use std::time::Duration;

use futures::StreamExt;
use tokio::{fs, io::AsyncWriteExt};

use crate::{
    downloader::DownloadTask,
    infra,
    providers::{
        ProgressCallback, ProviderError,
        soundcloud::{ClientBuilder, SoundCloudProvider, resolve::StreamSource},
    },
};

impl SoundCloudProvider {
    pub async fn download(
        &self,
        task: &DownloadTask,
        stream_source: StreamSource,
        on_progress: Option<ProgressCallback>,
    ) -> Result<(), ProviderError> {
        let max_retries = self.settings.read().await.system.max_retries;
        let mut attempts_left = max_retries.max(1);

        loop {
            let result = match &stream_source {
                StreamSource::Progressive { url, proxy_url } => {
                    self.download_progressive(task, url, proxy_url.as_deref(), on_progress.as_ref())
                        .await
                }
                StreamSource::Hls { url, proxy_url } => {
                    self.download_hls(task, url, proxy_url.as_deref(), on_progress.as_ref())
                        .await
                }
            };

            match result {
                Ok(()) => return Ok(()),
                Err(err) => {
                    attempts_left -= 1;
                    if attempts_left == 0 {
                        return Err(err);
                    }
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }

    async fn download_progressive(
        &self,
        task: &DownloadTask,
        url: &str,
        proxy_url: Option<&str>,
        on_progress: Option<&ProgressCallback>,
    ) -> Result<(), ProviderError> {
        let active_proxy = proxy_url
            .map(String::from)
            .or_else(|| self.settings.get_proxy_url_sync());
        let client = self.http_clients.get_or_create(active_proxy);
        let response = client.get(url).send().await?.error_for_status()?;
        let exact_size = response.content_length();
        let total = exact_size.unwrap_or_else(|| match task.track.duration_ms {
            Some(ms) => ((ms as f64 / 1000.0).max(1.0) * 16_000.0) as u64,
            None => 2_500_000,
        });

        let mut file = fs::File::create(&task.file_path).await?;
        let mut stream = response.bytes_stream();
        let mut position = 0;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            position += chunk.len() as u64;

            if let Some(cb) = on_progress {
                cb(position, total);
            }
        }

        drop(file);
        infra::verify_audio_file(&task.file_path, exact_size.unwrap_or(0)).await?;
        Ok(())
    }

    async fn download_hls(
        &self,
        task: &DownloadTask,
        stream_url: &str,
        proxy_url: Option<&str>,
        on_progress: Option<&ProgressCallback>,
    ) -> Result<(), ProviderError> {
        let (active_proxy, settings_snapshot) = {
            let settings = self.settings.read().await;
            let proxy = proxy_url
                .map(String::from)
                .or_else(|| settings.network.get_proxy_url().map(String::from));
            (proxy, settings.clone())
        };

        let proxied_client;
        let client: &soundcloud_rs::Client = if let Some(proxy) = active_proxy.as_deref() {
            proxied_client = ClientBuilder::new(&settings_snapshot)
                .with_proxy(Some(proxy))
                .build()
                .await?;
            &proxied_client
        } else {
            &self.client
        };

        let path_clone = task.file_path.clone();
        let duration_ms = task.track.duration_ms;
        let progress_cb = on_progress.cloned();

        let progress_task = tokio::spawn(async move {
            let mut has_existed = false;
            let mut missing_count = 0;
            loop {
                tokio::time::sleep(Duration::from_millis(500)).await;
                match fs::metadata(&path_clone).await {
                    Ok(meta) => {
                        has_existed = true;
                        missing_count = 0;
                        if let Some(ref cb) = progress_cb {
                            let estimated_total = match duration_ms {
                                Some(ms) => ((ms as f64 / 1000.0).max(1.0) * 24_000.0) as u64,
                                None => 3_500_000,
                            };
                            cb(meta.len(), estimated_total);
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

        download_res?;
        infra::verify_audio_file(&task.file_path, 0).await?;
        Ok(())
    }
}

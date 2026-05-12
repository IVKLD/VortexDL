use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::{Result, anyhow};
use colored::Colorize;
use soundcloud_rs::{Identifier, StreamType};
use tokio::{sync::RwLock, task::JoinHandle, time::sleep};

use crate::{
    api::download_manager::{DownloadManager, DownloadStatus},
    database::settings::UserSettings,
    downloader::core::{
        artwork::start_artwork_download, hls::try_download_hls,
        progressive::try_download_progressive,
    },
    storage::MusicStorage,
    utils::metadata::{SaveTrackArgs, save_track_info},
};

pub(in crate::downloader) struct Context<'a> {
    pub client: &'a soundcloud_rs::Client,
    pub http: &'a reqwest::Client,
    pub storage: &'a Arc<RwLock<MusicStorage>>,
    pub dm: Option<&'a Arc<DownloadManager>>,
    pub settings: &'a Arc<RwLock<UserSettings>>,
}

pub(in crate::downloader) struct Task<'a> {
    pub id: i64,
    pub filename: String,
    pub artwork_url: Option<&'a str>,
    pub pb: &'a indicatif::ProgressBar,
    pub output_dir: String,
    pub file_path: String,
}

pub(in crate::downloader) async fn initiate_track_download(
    ctx: &Context<'_>,
    task: &Task<'_>,
) -> Option<JoinHandle<()>> {
    if let Some(m) = ctx.dm {
        m.update_status(task.id, DownloadStatus::Downloading).await;
    }

    task.pb
        .set_message(format!("Downloading Music & Art: {}", task.filename));

    let artwork_url = task
        .artwork_url
        .map(|url| url.replace("-large", "-t1080x1080"));
    let artwork_task = artwork_url
        .as_ref()
        .map(|url| start_artwork_download(ctx.http, url.clone()));

    match run_download(ctx, task).await {
        Ok(source_url) => {
            task.pb
                .println(format!("{} Done: {}", "[OK]".green().bold(), task.filename));

            let storage = Arc::clone(ctx.storage);
            let dm = ctx.dm.map(Arc::clone);
            let id = task.id;
            let file_path = task.file_path.clone();

            Some(tokio::spawn(async move {
                let art_data = if let Some(handle) = artwork_task {
                    handle.await.unwrap_or(None)
                } else {
                    None
                };

                let file_path_clone = file_path.clone();
                let sc_id_str = id.to_string();
                let artwork_url_clone = artwork_url.clone();
                let source_url_clone = source_url.clone();

                let _ = tokio::task::spawn_blocking(move || {
                    let args = SaveTrackArgs {
                        path: &file_path_clone,
                        sc_id: &sc_id_str,
                        artwork_url: artwork_url_clone.as_deref(),
                        source_url: source_url_clone.as_deref(),
                        artwork_data: art_data,
                    };
                    save_track_info(args).ok();
                })
                .await;

                storage.write().await.update_track(
                    id,
                    PathBuf::from(&file_path),
                    artwork_url,
                    source_url.clone(),
                );

                if let Some(m) = dm {
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    let size = std::fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0);
                    m.update_finished(id, "mp3".to_string(), now, source_url, size)
                        .await;
                }
            }))
        }
        Err(e) => {
            if let Some(m) = ctx.dm {
                m.update_status(task.id, DownloadStatus::Failed).await;
            }
            task.pb.println(format!(
                "{} Failed: {} — {:#}",
                "[ERROR]".red().bold(),
                task.filename,
                e
            ));
            None
        }
    }
}

async fn run_download(ctx: &Context<'_>, task: &Task<'_>) -> Result<Option<String>> {
    let sc_id = Identifier::Id(task.id);
    let track = ctx.client.get_track(&sc_id).await?;
    let source_url = track.permalink_url.clone();

    let transcodings = track.media.as_ref().and_then(|m| m.transcodings.as_ref());
    let has_progressive = transcodings.is_some_and(|t| {
        t.iter().any(|tr| {
            tr.format.as_ref().and_then(|f| f.protocol.as_ref()) == Some(&StreamType::Progressive)
        })
    });
    let has_hls = transcodings.is_some_and(|t| {
        t.iter().any(|tr| {
            tr.format.as_ref().and_then(|f| f.protocol.as_ref()) == Some(&StreamType::Hls)
        })
    });

    if !has_progressive && !has_hls {
        return Err(anyhow!("No downloadable streams available for this track"));
    }

    let mut retries = {
        let s = ctx.settings.read().await;
        s.max_retries
    };
    let mut last_err = None;

    while retries > 0 {
        let download_result = if has_progressive {
            try_download_progressive(ctx, task, &sc_id).await
        } else {
            try_download_hls(ctx, task, &track, &sc_id).await
        };

        match download_result {
            Ok(_) => return Ok(source_url),
            Err(e) => {
                last_err = Some(e);
                retries -= 1;

                if retries > 0 {
                    task.pb
                        .set_message(format!("Retrying ({} left): {}", retries, task.filename));
                    sleep(Duration::from_secs(2)).await;
                }
            }
        }
    }

    if let Some(e) = last_err {
        tokio::fs::remove_file(&task.file_path).await.ok();
        return Err(e);
    }

    Ok(source_url)
}

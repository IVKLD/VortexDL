pub mod artwork;
pub mod complete;
pub mod download;
pub mod resolve;

use std::path::PathBuf;

use tokio::task::JoinHandle;

use crate::downloader::Context;

#[derive(Clone)]
pub struct DownloadTask {
    pub id: i64,
    pub title: String,
    pub artist: String,
    pub artwork_url: Option<String>,
    pub position: Option<u32>,
    pub pb: indicatif::ProgressBar,
    pub file_path: PathBuf,
}

impl DownloadTask {
    pub fn display_name(&self) -> &str {
        self.file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&self.title)
    }
}

pub async fn run_track_pipeline(ctx: Context, mut task: DownloadTask) -> Option<JoinHandle<()>> {
    let pipeline = async {
        if let Some(m) = &ctx.dm {
            m.update_downloading(task.id);
        }
        let display_name = task.display_name().to_string();
        task.pb.set_message(format!("Downloading: {display_name}"));

        let (track, sc_id, proto) = resolve::resolve_track_metadata(&ctx, task.id).await?;
        let artwork_handle = artwork::spawn_artwork_download(&ctx, &mut task);
        let url = proto.url().to_string();

        download::download_with_retries(&ctx, &task, &track, &sc_id, proto).await?;

        Ok((artwork_handle, url))
    };

    match pipeline.await {
        Ok((artwork_handle, url)) => Some(complete::finalize_and_persist(
            ctx,
            task,
            artwork_handle,
            url,
        )),
        Err(e) => {
            complete::on_failure(&ctx, &task, e).await;
            None
        }
    }
}

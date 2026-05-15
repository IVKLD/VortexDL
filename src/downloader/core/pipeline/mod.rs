use tokio::task::JoinHandle;

pub mod artwork;
pub mod complete;
pub mod download;
pub mod prepare;
pub mod resolve;

use crate::downloader::Context;

#[derive(Clone)]
pub struct DownloadTask {
    pub id: i64,
    pub title: String,
    pub artist: String,
    pub artwork_url: Option<String>,
    pub position: Option<u32>,
    pub pb: indicatif::ProgressBar,
    pub output_dir: String,
    pub file_path: String,
}

/// Orchestrates the track download pipeline.
pub async fn run_track_pipeline(ctx: Context, mut task: DownloadTask) -> Option<JoinHandle<()>> {
    let pipeline = async {
        prepare::prepare_environment(&ctx, &task).await;
        let artwork_handle = artwork::spawn_artwork_download(&ctx, &mut task);

        let (track, sc_id, proto) = resolve::resolve_track_metadata(&ctx, task.id).await?;

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

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

pub async fn run_track_pipeline(context: Context, task: DownloadTask) -> Option<JoinHandle<()>> {
    let pipeline = async {
        if let Some(manager) = &context.dm {
            manager.update_downloading(task.id);
        }
        let display_name = task.display_name().to_string();
        task.pb.set_message(format!("Downloading: {display_name}"));

        let proto = resolve::resolve_stream_source(&context, task.id).await?;

        let artwork_handle = resolve::spawn_artwork_fetch(&context, task.artwork_url.as_deref());

        let url = proto.url().to_string();

        download::download(&context, &task, proto).await?;

        Ok((artwork_handle, url))
    };

    match pipeline.await {
        Ok((artwork_handle, url)) => Some(tokio::spawn(complete::finalize(
            context,
            task,
            artwork_handle,
            url,
        ))),
        Err(err) => {
            complete::fail(&context, &task, err).await;
            None
        }
    }
}

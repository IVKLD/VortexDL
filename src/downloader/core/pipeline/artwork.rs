use tokio::task::JoinHandle;

use crate::{
    downloader::{Context, core::pipeline::DownloadTask},
    utils::soundcloud::fetch_artwork,
};

pub type ArtworkDataHandle = JoinHandle<Option<Vec<u8>>>;

pub fn spawn_artwork_download(ctx: &Context, task: &mut DownloadTask) -> Option<ArtworkDataHandle> {
    let url = task.artwork_url.as_ref()?;

    let high_res_url = url.replace("-large", "-t1080x1080");
    let http = ctx.http.clone();
    let download_url = high_res_url.clone();
    task.artwork_url = Some(high_res_url);

    Some(tokio::spawn(async move {
        fetch_artwork(&http, &download_url).await
    }))
}

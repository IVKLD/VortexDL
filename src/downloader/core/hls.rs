use anyhow::{Result, anyhow};
use soundcloud_rs::{Identifier, StreamType, Track};

use crate::downloader::core::{
    track::{Context, Task},
    verification::verify_file,
};

pub(super) async fn try_download_hls(
    ctx: &Context<'_>,
    task: &Task<'_>,
    track: &Track,
    sc_id: &Identifier,
) -> Result<()> {
    let filename = task.filename();

    task.pb
        .set_message(format!("Downloading Music & Art (HLS): {}", filename));

    ctx.client
        .download_track(
            track,
            sc_id,
            Some(&StreamType::Hls),
            Some(&task.output_dir),
            Some(&filename),
        )
        .await
        .map_err(|e| anyhow!("HLS download failed: {}", e))?;

    verify_file(&task.file_path, 0).await
}

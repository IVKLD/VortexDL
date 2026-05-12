use anyhow::{Result, anyhow};
use futures::StreamExt;
use soundcloud_rs::{Identifier, StreamType};
use tokio::{fs, io::AsyncWriteExt};

use crate::{
    downloader::core::{
        track::{Context, Task},
        verification::verify_file,
    },
    ui,
};

pub(super) async fn try_download_progressive(
    ctx: &Context<'_>,
    task: &Task<'_>,
    sc_id: &Identifier,
) -> Result<()> {
    let url = ctx
        .client
        .get_stream_url(sc_id, Some(&StreamType::Progressive))
        .await?;

    let response = ctx.http.get(&url).send().await?.error_for_status()?;
    let total = response.content_length().unwrap_or(0);

    task.pb
        .set_message(format!("Downloading Music & Art: {}", task.filename()));

    ui::upgrade_to_download_bar(task.pb, total);

    let mut file = fs::File::create(&task.file_path).await?;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| anyhow!("Stream error: {}", e))?;
        file.write_all(&chunk).await?;

        let len = chunk.len() as u64;
        task.pb.inc(len);

        if let Some(m) = ctx.dm {
            m.update_progress(task.id, task.pb.position(), total).await;
        }
    }

    drop(file);
    verify_file(&task.file_path, total).await?;

    Ok(())
}

use crate::{
    api::download_manager::DownloadStatus,
    downloader::{Context, core::pipeline::DownloadTask},
    utils::filename::format_track_filename,
};

/// Prepares the track environment for download.
pub async fn prepare_environment(ctx: &Context, task: &DownloadTask) {
    if let Some(m) = &ctx.dm {
        m.update_status(task.id, DownloadStatus::Downloading);
    }

    task.pb.set_message(format!(
        "Downloading Music & Art: {}",
        format_track_filename(&task.artist, &task.title)
    ));
}

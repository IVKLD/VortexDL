use std::{collections::HashSet, sync::Arc};

use anyhow::Result;
use tokio::sync::RwLock;

use super::{
    commands::{ensure_remote_dir, list_remote_files, run_adb_push, run_adb_rm},
    ui::build_progress_bar,
};
use crate::storage::MusicStorage;

pub async fn sync_device(
    device_id: &str,
    remote_dir: &str,
    storage: Arc<RwLock<MusicStorage>>,
) -> Result<()> {
    let local_tracks = storage.read().await.tracks.clone();

    if local_tracks.is_empty() {
        tracing::debug!(device = %device_id, "No local tracks, skipping sync");
        return Ok(());
    }

    ensure_remote_dir(device_id, remote_dir).await?;

    let remote_files = list_remote_files(device_id, remote_dir).await?;

    let local_filenames: HashSet<String> = local_tracks
        .iter()
        .filter_map(|(id, track)| {
            let ext = track.path.extension()?.to_str()?;
            Some(format!("{id}.{ext}"))
        })
        .collect();

    let to_push: Vec<_> = local_tracks
        .iter()
        .filter_map(|(id, track)| {
            let ext = track.path.extension()?.to_str()?;
            let filename = format!("{id}.{ext}");
            (!remote_files.contains(&filename)).then_some((*id, filename, track.clone()))
        })
        .collect();

    let to_delete: Vec<String> = remote_files
        .iter()
        .filter(|f| !local_filenames.contains(*f))
        .cloned()
        .collect();

    if to_push.is_empty() && to_delete.is_empty() {
        tracing::info!(
            device = %device_id,
            total = local_tracks.len(),
            dir = remote_dir,
            "Already in sync, nothing to do"
        );
        return Ok(());
    }

    if !to_delete.is_empty() {
        tracing::info!(device = %device_id, count = to_delete.len(), "Removing orphaned tracks from device");
        delete_remote_tracks(device_id, remote_dir, &to_delete).await;
    }

    if !to_push.is_empty() {
        tracing::info!(device = %device_id, count = to_push.len(), dir = remote_dir, "Starting push");
        push_tracks(device_id, remote_dir, to_push).await?;
    }

    Ok(())
}

async fn push_tracks(
    device_id: &str,
    remote_dir: &str,
    tracks: Vec<(i64, String, crate::storage::TrackData)>,
) -> Result<()> {
    let total = tracks.len() as u64;
    let pb = build_progress_bar(total, device_id);

    let mut pushed = 0u64;
    let mut failed = 0u64;

    for (id, filename, track) in tracks {
        let Some(local_path) = track.path.to_str() else {
            tracing::warn!(device = %device_id, track_id = %id, "Track path is not valid UTF-8, skipping");
            failed += 1;
            pb.inc(1);
            continue;
        };

        let remote_path = format!("{remote_dir}/{filename}");
        pb.set_message(filename.clone());

        match run_adb_push(device_id, local_path, &remote_path).await {
            Ok(true) => {
                pushed += 1;
                tracing::debug!(device = %device_id, track_id = %id, file = %filename, "Pushed");
            }
            Ok(false) => {
                failed += 1;
                tracing::warn!(device = %device_id, track_id = %id, "Failed to push track");
            }
            Err(e) => {
                failed += 1;
                tracing::error!(device = %device_id, track_id = %id, error = %e, "adb push command error");
            }
        }

        pb.inc(1);
    }

    pb.finish_with_message(format!("✓ {pushed} pushed, {failed} failed"));
    tracing::info!(device = %device_id, pushed, failed, "Sync complete");

    Ok(())
}

async fn delete_remote_tracks(device_id: &str, remote_dir: &str, filenames: &[String]) {
    for filename in filenames {
        let remote_path = format!("{remote_dir}/{filename}");

        match run_adb_rm(device_id, &remote_path).await {
            Ok(true) => {
                tracing::info!(device = %device_id, file = %filename, "Deleted orphaned track");
            }
            Ok(false) => {
                tracing::warn!(device = %device_id, file = %filename, "adb rm returned non-zero");
            }
            Err(e) => {
                tracing::error!(device = %device_id, file = %filename, error = %e, "Failed to run adb rm");
            }
        }
    }
}

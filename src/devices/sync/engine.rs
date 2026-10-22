use std::{collections::HashSet, path::PathBuf, sync::Arc, time::Duration};

use anyhow::Result;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::{
    devices::{
        error::DeviceError,
        files,
        registry::{self, SyncGuard},
    },
    domain,
    library::Library,
    terminal,
};

struct PushTask {
    rel_path: String,
    artist_dir: String,
    local_path: PathBuf,
}

struct SyncDiff {
    to_push: Vec<PushTask>,
    to_delete: Vec<String>,
}

fn diff_local_and_remote(storage: &Library, remote_files: &HashSet<String>) -> SyncDiff {
    let mut local_paths_lower = HashSet::new();
    let mut to_push = Vec::new();

    let mut remote_lower = HashSet::new();
    for rf in remote_files {
        remote_lower.insert(rf.to_lowercase());
    }

    for track in storage.values() {
        if track.is_archived {
            continue;
        }
        let Some(name) = track.path.file_name().and_then(|f| f.to_str()) else {
            continue;
        };
        let artist_dir = domain::clean_filename(&track.metadata.artist);
        let rel = format!("{artist_dir}/{name}");
        local_paths_lower.insert(rel.to_lowercase());

        if !remote_lower.contains(&rel.to_lowercase()) {
            to_push.push(PushTask {
                rel_path: rel,
                artist_dir,
                local_path: track.path.clone(),
            });
        }
    }

    let to_delete = remote_files
        .iter()
        .filter(|f| !local_paths_lower.contains(&f.to_lowercase()))
        .cloned()
        .collect();

    SyncDiff { to_push, to_delete }
}

/// What: Synchronizes all local music tracks to a target Android device over ADB.
/// Flow: Triggered manually from CLI/WebUI or automatically by the device tracker when an authorized device connects.
/// Why:  Avoids redundant uploads: it queries remote files, computes the diff against local storage,
///       pushes missing tracks, deletes orphaned ones, and uses SyncGuard to guarantee exclusive device access.
pub async fn sync_device(
    device: &str,
    remote_dir: &str,
    storage: Arc<RwLock<Library>>,
    force: bool,
) -> Result<(), DeviceError> {
    if !force
        && registry::get_last_sync_time(device)
            .is_some_and(|last_sync| last_sync.elapsed() < Duration::from_secs(10))
    {
        return Ok(());
    }

    let _guard = match SyncGuard::try_acquire(device) {
        Some(g) => g,
        None => {
            if force {
                return Err(DeviceError::AlreadyInProgress);
            }
            return Ok(());
        }
    };

    if let Err(e) = files::ensure_dir(device, remote_dir).await {
        error!("Cannot access remote directory '{remote_dir}' on device {device}: {e}");
        return Err(e);
    }

    let remote_files = files::list_files(device, remote_dir).await?;

    let diff = {
        let storage_read = storage.read().await;
        if storage_read.is_empty() {
            return Ok(());
        }
        diff_local_and_remote(&storage_read, &remote_files)
    };

    if diff.to_push.is_empty() && diff.to_delete.is_empty() {
        info!("[{device}] Sync not needed, no changes");
        registry::update_last_sync_time(device);
        return Ok(());
    }

    info!("[{device}] Syncing ADB device -> {remote_dir}");

    if !diff.to_delete.is_empty() {
        info!(
            "[{device}] Removing {} orphaned tracks",
            diff.to_delete.len()
        );
        delete_tracks(device, remote_dir, &diff.to_delete).await;
        let _ = files::sync_device_fs(device).await;
    }

    if !diff.to_push.is_empty() {
        info!("[{device}] Pushing {} tracks", diff.to_push.len());
        push_tracks(device, remote_dir, diff.to_push).await?;
        let _ = files::sync_device_fs(device).await;
    }

    registry::update_last_sync_time(device);
    info!("[{device}] Sync complete");

    Ok(())
}

async fn push_tracks(
    device: &str,
    remote_dir: &str,
    tracks: Vec<PushTask>,
) -> Result<(), DeviceError> {
    let pb = terminal::create_device_progress_bar(tracks.len() as u64, device);
    let (mut pushed, mut failed) = (0u64, 0u64);

    for task in &tracks {
        let Some(local) = task.local_path.to_str() else {
            terminal::pb_warn(&pb, format!("Invalid path: {}", task.local_path.display()));
            failed += 1;
            pb.inc(1);
            continue;
        };

        let parent = format!("{remote_dir}/{}", task.artist_dir);
        if let Err(e) = files::ensure_dir(device, &parent).await {
            terminal::pb_err(&pb, format!("mkdir {parent}: {e}"));
            failed += 1;
            pb.inc(1);
            continue;
        };

        let remote = format!("{remote_dir}/{}", task.rel_path);
        pb.set_message(task.rel_path.clone());

        match files::push(device, local, &remote).await {
            Ok(()) => {
                pushed += 1;
            }
            Err(e) => {
                failed += 1;
                terminal::pb_err(&pb, format!("push {}: {e}", task.rel_path));
            }
        }

        pb.inc(1);
    }

    pb.finish_and_clear();
    if failed > 0 {
        warn!("Track push completed with failures (pushed: {pushed}, failed: {failed})");
    } else {
        info!("Track push completed successfully ({pushed} tracks)");
    }
    Ok(())
}

async fn delete_tracks(device: &str, remote_dir: &str, paths: &[String]) {
    let mut parents_to_clean = HashSet::new();

    for rel in paths {
        let remote = format!("{remote_dir}/{rel}");

        match files::delete_file(device, &remote).await {
            Ok(()) => {
                info!("Deleted file: {rel}");
                if let Some((parent, _)) = rel.split_once('/') {
                    parents_to_clean.insert(parent.to_string());
                }
            }
            Err(e) => warn!("Failed to delete file {rel}: {e}"),
        }
    }

    for parent in parents_to_clean {
        let _ = files::delete_dir(device, &format!("{remote_dir}/{parent}")).await;
    }
}

use std::{collections::HashSet, path::PathBuf, sync::Arc, time::Duration};

use anyhow::Result;
use tokio::sync::RwLock;

use super::{
    commands,
    state::{SyncGuard, get_last_sync_time, update_last_sync_time},
    ui,
};
use crate::{storage::MusicStorage, utils::filename::clean_filename};

struct PushTask {
    rel_path: String,
    artist_dir: String,
    local_path: PathBuf,
}

struct SyncDiff {
    to_push: Vec<PushTask>,
    to_delete: Vec<String>,
}

fn diff_local_and_remote(storage: &MusicStorage, remote_files: &HashSet<String>) -> SyncDiff {
    let mut local_paths = HashSet::new();
    let mut to_push = Vec::new();

    for track in storage.tracks.values() {
        if track.is_archived() {
            continue;
        }
        let Some(name) = track.path.file_name().and_then(|f| f.to_str()) else {
            continue;
        };
        let artist_dir = clean_filename(&track.metadata.artist);
        let rel = format!("{artist_dir}/{name}");
        local_paths.insert(rel.clone());

        if !remote_files.contains(&rel) {
            to_push.push(PushTask {
                rel_path: rel,
                artist_dir,
                local_path: track.path.clone(),
            });
        }
    }

    let to_delete = remote_files
        .iter()
        .filter(|f| !local_paths.contains(*f))
        .cloned()
        .collect();

    SyncDiff { to_push, to_delete }
}

pub async fn sync_device(
    device: &str,
    remote_dir: &str,
    storage: Arc<RwLock<MusicStorage>>,
) -> Result<()> {
    if get_last_sync_time(device)
        .is_some_and(|last_sync| last_sync.elapsed() < Duration::from_secs(10))
    {
        return Ok(());
    }

    let _guard = match SyncGuard::try_acquire(device) {
        Some(g) => g,
        None => return Ok(()),
    };

    if let Err(e) = commands::ensure_dir(device, remote_dir).await {
        ui::remote_access_failed(remote_dir, device, &e);
        return Err(e);
    }

    let _ = commands::sync_device_fs(device).await;

    let remote_files = commands::list_files(device, remote_dir).await?;

    let diff = {
        let storage_read = storage.read().await;
        if storage_read.tracks.is_empty() {
            return Ok(());
        }
        diff_local_and_remote(&storage_read, &remote_files)
    };

    if diff.to_push.is_empty() && diff.to_delete.is_empty() {
        ui::sync_not_needed(device);
        update_last_sync_time(device);
        return Ok(());
    }

    ui::sync_start(device, remote_dir);

    if !diff.to_delete.is_empty() {
        ui::removing(diff.to_delete.len(), device);
        delete_tracks(device, remote_dir, &diff.to_delete).await;
        let _ = commands::sync_device_fs(device).await;
    }

    if !diff.to_push.is_empty() {
        ui::pushing(diff.to_push.len(), device);
        push_tracks(device, remote_dir, diff.to_push).await?;
        let _ = commands::sync_device_fs(device).await;
    }

    update_last_sync_time(device);

    ui::sync_complete(device);

    Ok(())
}

async fn push_tracks(device: &str, remote_dir: &str, tracks: Vec<PushTask>) -> Result<()> {
    let pb = ui::progress_bar(tracks.len() as u64, device);
    let (mut pushed, mut failed) = (0u64, 0u64);

    for task in &tracks {
        let Some(local) = task.local_path.to_str() else {
            ui::pb_warn(&pb, format!("Invalid path: {}", task.local_path.display()));
            failed += 1;
            pb.inc(1);
            continue;
        };

        let parent = format!("{remote_dir}/{}", task.artist_dir);
        if let Err(e) = commands::ensure_dir(device, &parent).await {
            ui::pb_err(&pb, format!("mkdir {parent}: {e}"));
            failed += 1;
            pb.inc(1);
            continue;
        }

        let remote = format!("{remote_dir}/{}", task.rel_path);
        pb.set_message(task.rel_path.clone());

        match commands::push(device, local, &remote).await {
            Ok(()) => {
                pushed += 1;
            }
            Err(e) => {
                failed += 1;
                ui::pb_err(&pb, format!("push {}: {e}", task.rel_path));
            }
        }

        pb.inc(1);
    }

    pb.finish_and_clear();
    ui::push_results(pushed, failed);
    Ok(())
}

async fn delete_tracks(device: &str, remote_dir: &str, paths: &[String]) {
    let mut parents_to_clean = HashSet::new();

    for rel in paths {
        let remote = format!("{remote_dir}/{rel}");

        match commands::delete_file(device, &remote).await {
            Ok(()) => {
                ui::deleted(rel);
                if let Some((parent, _)) = rel.split_once('/') {
                    parents_to_clean.insert(parent.to_string());
                }
            }
            Err(e) => ui::delete_failed(rel, &e),
        }
    }

    for parent in parents_to_clean {
        let _ = commands::delete_dir(device, &format!("{remote_dir}/{parent}")).await;
    }
}

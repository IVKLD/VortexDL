use std::{collections::HashSet, sync::Arc};

use anyhow::Result;
use tokio::sync::RwLock;

use super::{
    commands::{
        ensure_remote_dir, list_remote_files, run_adb_push, run_adb_rm, run_adb_rmdir,
        trigger_directory_scan, trigger_media_scan,
    },
    state::SyncGuard,
    ui::{self, build_progress_bar},
};
use crate::{storage::MusicStorage, utils::filename::clean_filename};

pub async fn sync_device(
    device_id: &str,
    remote_dir: &str,
    storage: Arc<RwLock<MusicStorage>>,
) -> Result<()> {
    let _guard = match SyncGuard::try_acquire(device_id) {
        Some(g) => g,
        None => {
            tracing::debug!(device = %device_id, "Sync already in progress, ignoring duplicate request");
            return Ok(());
        }
    };

    let local_tracks = storage.read().await.tracks.clone();

    if local_tracks.is_empty() {
        return Ok(());
    }

    if let Err(e) = ensure_remote_dir(device_id, remote_dir).await {
        ui::print_err_access_remote(remote_dir, device_id, &e);
        return Err(e);
    }

    let remote_files = list_remote_files(device_id, remote_dir).await?;

    let mut local_rel_paths = HashSet::new();
    let mut tracks_to_sync = Vec::new();

    for (id, track) in &local_tracks {
        let Some(file_name) = track.path.file_name().and_then(|f| f.to_str()) else {
            continue;
        };
        let artist_dir = clean_filename(&track.artist);
        let rel_path = format!("{artist_dir}/{file_name}");
        local_rel_paths.insert(rel_path.clone());

        if !remote_files.contains(&rel_path) {
            tracks_to_sync.push((*id, rel_path, artist_dir, track.clone()));
        }
    }

    let to_delete: Vec<String> = remote_files
        .iter()
        .filter(|f| !local_rel_paths.contains(*f))
        .cloned()
        .collect();

    if tracks_to_sync.is_empty() && to_delete.is_empty() {
        return Ok(());
    }

    ui::print_sync_start(device_id, remote_dir);

    if !to_delete.is_empty() {
        ui::print_removing_orphaned(to_delete.len(), device_id);
        delete_remote_tracks(device_id, remote_dir, &to_delete).await;
    }

    if !tracks_to_sync.is_empty() {
        ui::print_pushing_new(tracks_to_sync.len(), device_id);
        push_tracks(device_id, remote_dir, tracks_to_sync).await?;
    }

    let _ = trigger_directory_scan(device_id, remote_dir).await;

    ui::print_sync_complete(device_id);

    Ok(())
}

async fn push_tracks(
    device_id: &str,
    remote_dir: &str,
    tracks: Vec<(i64, String, String, crate::storage::TrackData)>,
) -> Result<()> {
    let total = tracks.len() as u64;
    let pb = build_progress_bar(total, device_id);

    let mut pushed = 0u64;
    let mut failed = 0u64;

    for (id, rel_path, artist_dir, track) in tracks {
        let Some(local_path) = track.path.to_str() else {
            ui::log_warn_invalid_path(&pb, id);
            failed += 1;
            pb.inc(1);
            continue;
        };

        let remote_parent_dir = format!("{remote_dir}/{artist_dir}");
        if let Err(e) = ensure_remote_dir(device_id, &remote_parent_dir).await {
            ui::log_err_create_artist_dir(&pb, &remote_parent_dir, &e);
            failed += 1;
            pb.inc(1);
            continue;
        }

        let remote_path = format!("{remote_dir}/{rel_path}");
        let display_name = rel_path.clone();
        pb.set_message(display_name.clone());

        match run_adb_push(device_id, local_path, &remote_path).await {
            Ok(()) => {
                pushed += 1;
                if let Err(e) = trigger_media_scan(device_id, &remote_path).await {
                    ui::log_warn_media_scan(&pb, &display_name, &e);
                }
            }
            Err(e) => {
                failed += 1;
                ui::log_err_push(&pb, &display_name, &e);
            }
        }

        pb.inc(1);
    }

    pb.finish_and_clear();

    ui::print_push_results(pushed, failed);

    Ok(())
}

async fn delete_remote_tracks(device_id: &str, remote_dir: &str, rel_paths: &[String]) {
    for rel_path in rel_paths {
        let remote_path = format!("{remote_dir}/{rel_path}");

        match run_adb_rm(device_id, &remote_path).await {
            Ok(()) => {
                ui::print_deleted_orphaned(rel_path);
                if let Some((parent_dir, _)) = rel_path.split_once('/') {
                    let remote_parent_path = format!("{remote_dir}/{parent_dir}");
                    let _ = run_adb_rmdir(device_id, &remote_parent_path).await;
                }
            }
            Err(e) => {
                ui::print_fail_delete_orphaned(rel_path, &e);
            }
        }
    }
}

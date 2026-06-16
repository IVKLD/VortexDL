use std::{collections::HashSet, sync::Arc};

use anyhow::Result;
use tokio::sync::RwLock;

use super::{commands, state::SyncGuard, ui};
use crate::{storage::MusicStorage, utils::filename::clean_filename};

pub async fn sync_device(
    device: &str,
    remote_dir: &str,
    storage: Arc<RwLock<MusicStorage>>,
) -> Result<()> {
    let _guard = match SyncGuard::try_acquire(device) {
        Some(g) => g,
        None => return Ok(()),
    };

    if let Err(e) = commands::ensure_remote_dir(device, remote_dir).await {
        ui::remote_access_failed(remote_dir, device, &e);
        return Err(e);
    }

    let remote_files = commands::list_remote_files(device, remote_dir).await?;

    let mut local_paths = HashSet::new();
    let mut to_push = Vec::new();

    {
        let storage_read = storage.read().await;
        let tracks = &storage_read.tracks;
        if tracks.is_empty() {
            return Ok(());
        }

        for track in tracks.values() {
            let Some(name) = track.path.file_name().and_then(|f| f.to_str()) else {
                continue;
            };
            let artist_dir = clean_filename(&track.artist);
            let rel = format!("{artist_dir}/{name}");
            local_paths.insert(rel.clone());

            if !remote_files.contains(&rel) {
                to_push.push((rel, artist_dir, track.path.clone()));
            }
        }
    }

    let to_delete: Vec<_> = remote_files
        .iter()
        .filter(|f| !local_paths.contains(*f))
        .cloned()
        .collect();

    if to_push.is_empty() && to_delete.is_empty() {
        return Ok(());
    }

    ui::sync_start(device, remote_dir);

    if !to_delete.is_empty() {
        ui::removing(to_delete.len(), device);
        delete_tracks(device, remote_dir, &to_delete).await;
    }

    if !to_push.is_empty() {
        ui::pushing(to_push.len(), device);
        push_tracks(device, remote_dir, to_push).await?;
    }

    let _ = commands::dir_scan(device, remote_dir).await;
    ui::sync_complete(device);

    Ok(())
}

async fn push_tracks(
    device: &str,
    remote_dir: &str,
    tracks: Vec<(String, String, std::path::PathBuf)>,
) -> Result<()> {
    let pb = ui::progress_bar(tracks.len() as u64, device);
    let (mut pushed, mut failed) = (0u64, 0u64);

    for (rel, artist_dir, local_path) in &tracks {
        let Some(local) = local_path.to_str() else {
            ui::pb_warn(&pb, format!("Invalid path: {}", local_path.display()));
            failed += 1;
            pb.inc(1);
            continue;
        };

        let parent = format!("{remote_dir}/{artist_dir}");
        if let Err(e) = commands::ensure_remote_dir(device, &parent).await {
            ui::pb_err(&pb, format!("mkdir {parent}: {e}"));
            failed += 1;
            pb.inc(1);
            continue;
        }

        let remote = format!("{remote_dir}/{rel}");
        pb.set_message(rel.clone());

        match commands::push(device, local, &remote).await {
            Ok(()) => {
                pushed += 1;
                if let Err(e) = commands::media_scan(device, &remote).await {
                    ui::pb_warn(&pb, format!("media scan {rel}: {e}"));
                }
            }
            Err(e) => {
                failed += 1;
                ui::pb_err(&pb, format!("push {rel}: {e}"));
            }
        }

        pb.inc(1);
    }

    pb.finish_and_clear();
    ui::push_results(pushed, failed);
    Ok(())
}

async fn delete_tracks(device: &str, remote_dir: &str, paths: &[String]) {
    for rel in paths {
        let remote = format!("{remote_dir}/{rel}");

        match commands::rm(device, &remote).await {
            Ok(()) => {
                ui::deleted(rel);
                if let Some((parent, _)) = rel.split_once('/') {
                    let _ = commands::rmdir(device, &format!("{remote_dir}/{parent}")).await;
                }
            }
            Err(e) => ui::delete_failed(rel, &e),
        }
    }
}

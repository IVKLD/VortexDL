use std::{collections::HashSet, io, path::Path};

use tokio::task::spawn_blocking;

use super::{LocalMusicTrack, MusicStorage};
use crate::{
    constants::ARCHIVE_DIR,
    database::{cache::remove_cached_track, get_previous_ids, save_sync_ids},
    types::SyncMode,
};

impl MusicStorage {
    pub async fn remove_track(&mut self, id: i64) -> Result<Option<LocalMusicTrack>, io::Error> {
        let Some(data) = self.tracks.remove(&id) else {
            return Ok(None);
        };
        if data.path.exists() {
            tokio::fs::remove_file(&data.path).await?;
        }
        let path_str = data.path.to_string_lossy().into_owned();
        let _ = spawn_blocking(move || {
            let _ = remove_cached_track(&path_str);
        })
        .await;
        Ok(Some(data))
    }

    pub async fn sync_storage(
        &mut self,
        url: &str,
        current_soundcloud_ids: &HashSet<i64>,
        mode: &SyncMode,
    ) -> anyhow::Result<()> {
        let prev_ids = get_previous_ids(url)?;
        let to_remove: Vec<i64> = prev_ids
            .difference(current_soundcloud_ids)
            .copied()
            .collect();

        if to_remove.is_empty() || matches!(mode, SyncMode::Silent) {
            save_sync_ids(url, current_soundcloud_ids)?;
            return Ok(());
        }

        let archive_path = Path::new(&self.base_path).join(ARCHIVE_DIR);
        if matches!(mode, SyncMode::Archive) {
            tokio::fs::create_dir_all(&archive_path).await?;
        }

        for id in to_remove {
            match self.tracks.remove(&id) {
                Some(data) if data.path.exists() => match mode {
                    SyncMode::Full => {
                        tokio::fs::remove_file(&data.path).await?;
                    }
                    SyncMode::Archive => {
                        if let Some(name) = data.path.file_name() {
                            tokio::fs::rename(&data.path, archive_path.join(name)).await?;
                        }
                    }
                    _ => {}
                },
                Some(data) => {
                    tracing::warn!(
                        id,
                        path = %data.path.display(),
                        "Track removed from remote but file already missing on disk"
                    );
                }
                None => {}
            }
        }

        save_sync_ids(url, current_soundcloud_ids)?;
        Ok(())
    }
}

pub mod metadata;
mod model;
mod scanner;
pub mod sync;

use std::{
    collections::{HashMap, HashSet},
    io,
    path::Path,
};

use anyhow::Result;
pub use model::LocalMusicTrack;
use tokio::task::spawn_blocking;

use crate::database::cache::{remove_cached_track, update_cached_tracks_batch};

#[derive(Default)]
pub struct MusicStorage {
    pub tracks: HashMap<i64, LocalMusicTrack>,
}

impl MusicStorage {
    pub fn update_track(&mut self, id: i64, data: LocalMusicTrack) {
        self.tracks.insert(id, data);
    }

    pub fn update_tracks(&mut self, new_tracks: HashMap<i64, LocalMusicTrack>) {
        let count = new_tracks.len();
        self.tracks = new_tracks;
        tracing::debug!("Indexing complete. Found {} tracks.", count);
    }

    pub async fn remove_track(&mut self, id: i64) -> Result<Option<LocalMusicTrack>, io::Error> {
        let Some(data) = self.tracks.remove(&id) else {
            return Ok(None);
        };

        delete_track_file(&data.path).await?;
        let path_str = data.path.to_string_lossy().into_owned();
        let _ = spawn_blocking(move || {
            let _ = remove_cached_track(&path_str);
        })
        .await;
        Ok(Some(data))
    }

    pub async fn remove_tracks_batch(
        &mut self,
        ids: Vec<i64>,
    ) -> Result<Vec<LocalMusicTrack>, io::Error> {
        let mut removed = Vec::new();
        let mut to_remove_cache = HashSet::new();

        for id in ids {
            if let Some(data) = self.tracks.remove(&id) {
                delete_track_file(&data.path).await?;
                to_remove_cache.insert(data.path.to_string_lossy().into_owned());
                removed.push(data);
            }
        }

        if !to_remove_cache.is_empty() {
            spawn_blocking(move || {
                let _ = update_cached_tracks_batch(&HashMap::new(), &to_remove_cache);
            })
            .await?;
        }

        Ok(removed)
    }
}

async fn delete_track_file(path: &Path) -> Result<(), io::Error> {
    if path.exists() {
        tokio::fs::remove_file(path).await?;
    }
    Ok(())
}

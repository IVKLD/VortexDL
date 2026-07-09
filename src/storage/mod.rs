pub mod metadata;
mod model;
mod scanner;
mod sync;

use std::collections::{HashMap, HashSet};
use std::io;
use anyhow::Result;
use tokio::task::spawn_blocking;

pub use model::LocalMusicTrack;
use crate::{
    database::cache::{remove_cached_track, update_cached_tracks_batch},
};

#[derive(Default)]
pub struct MusicStorage {
    pub tracks: HashMap<i64, LocalMusicTrack>,
}

impl MusicStorage {
    pub fn update_track(&mut self, id: i64, data: LocalMusicTrack) {
        self.tracks.insert(id, data);
    }

    pub fn update_tracks(&mut self, mut new_tracks: HashMap<i64, LocalMusicTrack>) {
        for (id, data) in &self.tracks {
            if !new_tracks.contains_key(id) && data.path.exists() {
                new_tracks.insert(*id, data.clone());
            }
        }
        let count = new_tracks.len();
        self.tracks = new_tracks;
        tracing::debug!("Indexing complete. Found {} tracks.", count);
    }

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

    pub async fn remove_tracks_batch(
        &mut self,
        ids: Vec<i64>,
    ) -> Result<Vec<LocalMusicTrack>, io::Error> {
        let mut removed = Vec::new();
        let mut to_remove_cache = HashSet::new();

        for id in ids {
            if let Some(data) = self.tracks.remove(&id) {
                if data.path.exists() {
                    tokio::fs::remove_file(&data.path).await?;
                }
                let path_str = data.path.to_string_lossy().into_owned();
                to_remove_cache.insert(path_str);
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

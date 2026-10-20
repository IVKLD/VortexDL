use std::{
    collections::{HashMap, HashSet},
    io,
    path::Path,
};

use anyhow::Result;
use tokio::{fs::remove_file, task::spawn_blocking};

use super::types::LibraryTrack;
use crate::infra::db::{remove_cached_track, update_cached_tracks_batch};

#[derive(Default)]
pub struct Library {
    tracks: HashMap<i64, LibraryTrack>,
}

impl Library {
    pub fn get_track(&self, id: i64) -> Option<&LibraryTrack> {
        self.tracks.get(&id)
    }

    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    pub fn values(&self) -> impl Iterator<Item = &LibraryTrack> {
        self.tracks.values()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&i64, &LibraryTrack)> {
        self.tracks.iter()
    }

    pub fn is_track_downloaded(&self, id: i64) -> bool {
        self.tracks
            .get(&id)
            .is_some_and(|d| d.path.exists() && !d.is_archived)
    }

    pub fn update_track(&mut self, id: i64, data: LibraryTrack) {
        self.tracks.insert(id, data);
    }

    pub fn update_tracks(&mut self, new_tracks: HashMap<i64, LibraryTrack>) {
        let count = new_tracks.len();
        self.tracks = new_tracks;
        tracing::debug!("Indexing complete. Found {} tracks.", count);
    }

    pub async fn remove_track(&mut self, id: i64) -> Result<Option<LibraryTrack>, io::Error> {
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
    ) -> Result<Vec<LibraryTrack>, io::Error> {
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
        remove_file(path).await?;
    }
    Ok(())
}

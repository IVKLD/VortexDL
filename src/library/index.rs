use std::collections::HashMap;

use crate::library::types::LibraryTrack;

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

    pub fn remove_track(&mut self, id: i64) -> Option<LibraryTrack> {
        self.tracks.remove(&id)
    }

    pub fn remove_tracks(&mut self, ids: &[i64]) -> Vec<LibraryTrack> {
        let mut removed = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(track) = self.tracks.remove(id) {
                removed.push(track);
            }
        }
        removed
    }
}

mod index;
mod sync;
mod track;

use std::collections::HashMap;

pub use track::LocalMusicTrack;

#[derive(Default)]
pub struct MusicStorage {
    pub base_path: String,
    pub tracks: HashMap<i64, LocalMusicTrack>,
}

impl MusicStorage {
    pub fn new(base_path: String) -> Self {
        Self {
            base_path,
            tracks: HashMap::new(),
        }
    }

    pub fn update_track(&mut self, id: i64, data: LocalMusicTrack) {
        self.tracks.insert(id, data);
    }
}

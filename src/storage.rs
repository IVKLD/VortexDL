use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::{fs, path::{Path, PathBuf}};

use crate::models::{SC_IDENTIFIER, SC_ARTWORK_URL};
use crate::utils::audio_file_manipulator::get_mp3_custom_field;

#[derive(Default, Clone)]
pub struct TrackData {
    pub path: PathBuf,
    pub artwork_url: Option<String>,
}

#[derive(Default, Clone)]
pub struct MusicStorage {
    pub tracks: HashMap<i64, TrackData>,
}

impl MusicStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update_track(&mut self, id: i64, path: PathBuf, artwork_url: Option<String>) {
        self.tracks.insert(id, TrackData { path, artwork_url });
    }

    pub fn remove_track(&mut self, id: i64) {
        self.tracks.remove(&id);
    }

    pub fn indexing(&mut self, start_path: &Path) {
        let mut dirs_to_visit = vec![start_path.to_path_buf()];

        while let Some(current_dir) = dirs_to_visit.pop() {
            if let Ok(entries) = fs::read_dir(current_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();

                    if path.is_dir() {
                        dirs_to_visit.push(path);
                    } else if path.is_file() {
                        if let Some(path_str) = path.to_str() {
                            if let Some(content) = get_mp3_custom_field(path_str, SC_IDENTIFIER) {
                                if let Ok(id) = content.parse::<i64>() {
                                    let artwork_url = get_mp3_custom_field(path_str, SC_ARTWORK_URL);
                                    self.tracks.insert(id, TrackData { path, artwork_url });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub async fn sync_storage(
        &self,
        remote_ids: &HashSet<i64>,
        output_dir: &str,
        mode: &str,
    ) -> crate::models::Result<()> {
        let mut to_remove = Vec::new();

        for (id, data) in &self.tracks {
            if !remote_ids.contains(id) {
                to_remove.push((id, &data.path));
            }
        }

        if to_remove.is_empty() {
            return Ok(());
        }

        if mode == "full" {
            for (_, path) in to_remove {
                fs::remove_file(path)?;
            }
        } else {
            let archive_dir = Path::new(output_dir).join("Archive");
            fs::create_dir_all(&archive_dir)?;

            for (_, path) in to_remove {
                if let Some(file_name) = path.file_name() {
                    let dest = archive_dir.join(file_name);
                    fs::rename(path, dest)?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs::File;

    #[tokio::test]
    async fn test_indexing_and_sync() {
        let dir = tempdir().unwrap();
        let output_path = dir.path();

        let mut storage = MusicStorage::new();
        let track1_path = output_path.join("track1.mp3");
        let track2_path = output_path.join("track2.mp3");

        File::create(&track1_path).unwrap();
        File::create(&track2_path).unwrap();

        let mut remote_ids = HashSet::new();
        remote_ids.insert(1);

        storage.tracks.insert(1, TrackData { path: track1_path.clone(), artwork_url: None });
        storage.tracks.insert(2, TrackData { path: track2_path.clone(), artwork_url: None });

        storage.sync_storage(&remote_ids, output_path.to_str().unwrap(), "full").await.unwrap();

        assert!(track1_path.exists());
        assert!(!track2_path.exists());
    }

    #[tokio::test]
    async fn test_sync_archive() {
        let dir = tempdir().unwrap();
        let output_path = dir.path();

        let mut storage = MusicStorage::new();
        let track1_path = output_path.join("track1.mp3");
        File::create(&track1_path).unwrap();

        storage.tracks.insert(1, TrackData { path: track1_path.clone(), artwork_url: None });

        let remote_ids = HashSet::new();

        storage.sync_storage(&remote_ids, output_path.to_str().unwrap(), "archive").await.unwrap();

        assert!(!track1_path.exists());
        assert!(output_path.join("Archive").join("track1.mp3").exists());
    }
}

use std::collections::{HashMap, HashSet};
use std::{fs, path::{Path, PathBuf}};

use crate::models::{SC_IDENTIFIER, SC_ARTWORK_URL};
use crate::utils::metadata::read_custom_field;

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
    pub fn update_track(&mut self, id: i64, path: PathBuf, artwork_url: Option<String>) {
        self.tracks.insert(id, TrackData { path, artwork_url });
    }

    pub fn remove_track(&mut self, id: i64) {
        self.tracks.remove(&id);
    }

    pub fn indexing(&mut self, root: &Path) {
        let mut stack = vec![root.to_path_buf()];

        while let Some(dir) = stack.pop() {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        stack.push(path);
                    } else if path.is_file() {
                        if let Some(path_str) = path.to_str() {
                            if let Some(id_str) = read_custom_field(path_str, SC_IDENTIFIER) {
                                if let Ok(id) = id_str.parse::<i64>() {
                                    let artwork_url = read_custom_field(path_str, SC_ARTWORK_URL);
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
        let to_remove: Vec<_> = self.tracks.iter()
            .filter(|(id, _)| !remote_ids.contains(id))
            .collect();

        if to_remove.is_empty() { return Ok(()); }

        if mode == "full" {
            for (_, data) in to_remove {
                fs::remove_file(&data.path)?;
            }
        } else {
            let archive = Path::new(output_dir).join("Archive");
            fs::create_dir_all(&archive)?;

            for (_, data) in to_remove {
                if let Some(name) = data.path.file_name() {
                    fs::rename(&data.path, archive.join(name))?;
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
    async fn test_sync() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        let mut s = MusicStorage::default();
        let p1 = root.join("t1.mp3");
        let p2 = root.join("t2.mp3");

        File::create(&p1).unwrap();
        File::create(&p2).unwrap();

        s.tracks.insert(1, TrackData { path: p1.clone(), artwork_url: None });
        s.tracks.insert(2, TrackData { path: p2.clone(), artwork_url: None });

        let mut remote = HashSet::new();
        remote.insert(1);

        s.sync_storage(&remote, root.to_str().unwrap(), "full").await.unwrap();

        assert!(p1.exists());
        assert!(!p2.exists());
    }
}

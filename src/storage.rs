use std::collections::{HashMap, HashSet};
use std::{fs, path::{Path, PathBuf}};
use crate::models::{SC_IDENTIFIER, SC_ARTWORK_URL, SC_SOURCE_URL};
use crate::utils::metadata::read_custom_field;

#[derive(Default, Clone)]
pub struct TrackData {
    pub path: PathBuf,
    pub artwork_url: Option<String>,
    pub source_url: Option<String>,
}

#[derive(Default, Clone)]
pub struct MusicStorage {
    pub tracks: HashMap<i64, TrackData>,
}

impl MusicStorage {
    pub fn update_track(&mut self, id: i64, path: PathBuf, artwork_url: Option<String>, source_url: Option<String>) {
        self.tracks.insert(id, TrackData { path, artwork_url, source_url });
    }

    pub fn remove_track(&mut self, id: i64) {
        self.tracks.remove(&id);
    }

    pub fn indexing(&mut self, root: &Path) {
        let mut stack = vec![root.to_path_buf()];
        let mut seen_ids = HashSet::new();

        while let Some(dir) = stack.pop() {
            let Ok(entries) = fs::read_dir(dir) else { continue };

            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }

                if let Some(id) = path.to_str()
                    .and_then(|p| read_custom_field(p, SC_IDENTIFIER))
                    .and_then(|s| s.parse::<i64>().ok()) 
                {
                    seen_ids.insert(id);
                    let p_str = path.to_str().unwrap();
                    let artwork_url = read_custom_field(p_str, SC_ARTWORK_URL);
                    let source_url = read_custom_field(p_str, SC_SOURCE_URL);
                    
                    self.tracks.insert(id, TrackData {
                        path,
                        artwork_url,
                        source_url,
                    });
                }
            }
        }

        self.tracks.retain(|id, _| seen_ids.contains(id));
        tracing::info!("Indexing complete. Found {} tracks.", self.tracks.len());
    }

    pub async fn sync_storage(&self, remote_ids: &HashSet<i64>, output_dir: &str, mode: &str) -> anyhow::Result<()> {
        let to_remove: Vec<_> = self.tracks.iter().filter(|(id, _)| !remote_ids.contains(id)).collect();
        if to_remove.is_empty() { return Ok(()); }

        if mode == "full" {
            for (_, data) in to_remove {
                if data.path.exists() { fs::remove_file(&data.path)?; }
            }
        } else {
            let archive = Path::new(output_dir).join("Archive");
            fs::create_dir_all(&archive)?;
            for (_, data) in to_remove {
                if let Some(name) = data.path.file_name() {
                    if data.path.exists() { fs::rename(&data.path, archive.join(name))?; }
                }
            }
        }
        Ok(())
    }
}

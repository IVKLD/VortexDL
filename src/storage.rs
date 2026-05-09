use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;

use crate::{
    constants::{SC_ARTWORK_URL, SC_IDENTIFIER, SC_SOURCE_URL},
    database::sync::{get_previous_ids, save_sync_ids},
    models::SyncMode,
    utils::metadata::read_custom_field,
};

#[derive(Default, Clone)]
pub struct TrackData {
    pub path: PathBuf,
    pub artwork_url: Option<String>,
    pub source_url: Option<String>,
}

#[derive(Clone)]
pub struct MusicStorage {
    pub base_path: String,
    pub tracks: HashMap<i64, TrackData>,
}

impl MusicStorage {
    pub fn new(base_path: String) -> Self {
        Self {
            base_path,
            tracks: HashMap::new(),
        }
    }
    pub fn update_track(
        &mut self,
        id: i64,
        path: PathBuf,
        artwork_url: Option<String>,
        source_url: Option<String>,
    ) {
        self.tracks.insert(
            id,
            TrackData {
                path,
                artwork_url,
                source_url,
            },
        );
    }

    pub fn remove_track(&mut self, id: i64) {
        self.tracks.remove(&id);
    }

    pub fn indexing(&mut self, root: &Path) {
        let mut stack = vec![root.to_path_buf()];
        let mut seen_ids = HashSet::new();

        while let Some(dir) = stack.pop() {
            let Ok(entries) = fs::read_dir(dir) else {
                continue;
            };

            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }

                if let Some(id) = path
                    .to_str()
                    .and_then(|p| read_custom_field(p, SC_IDENTIFIER))
                    .and_then(|s| s.parse::<i64>().ok())
                {
                    seen_ids.insert(id);
                    let p_str = path.to_str().unwrap();
                    let artwork_url = read_custom_field(p_str, SC_ARTWORK_URL);
                    let source_url = read_custom_field(p_str, SC_SOURCE_URL);

                    self.tracks.insert(
                        id,
                        TrackData {
                            path,
                            artwork_url,
                            source_url,
                        },
                    );
                }
            }
        }

        self.tracks.retain(|id, _| seen_ids.contains(id));
        tracing::info!("Indexing complete. Found {} tracks.", self.tracks.len());
    }

    pub async fn sync_storage(
        &self,
        url: &str,
        remote_ids: &HashSet<i64>,
        mode: &SyncMode,
    ) -> Result<()> {
        let previous_ids = get_previous_ids(url)?;

        let to_remove: Vec<_> = previous_ids
            .iter()
            .filter(|id| !remote_ids.contains(id))
            .filter_map(|id| self.tracks.get(id))
            .collect();

        if to_remove.is_empty() || matches!(mode, SyncMode::Silent) {
            save_sync_ids(url, remote_ids)?;
            return Ok(());
        }

        let archive_path = Path::new(&self.base_path).join("Archive");
        if matches!(mode, SyncMode::Archive) {
            fs::create_dir_all(&archive_path)?;
        }

        for data in to_remove {
            if !data.path.exists() {
                continue;
            }

            match mode {
                SyncMode::Full => fs::remove_file(&data.path)?,
                SyncMode::Archive => {
                    if let Some(name) = data.path.file_name() {
                        fs::rename(&data.path, archive_path.join(name))?;
                    }
                }
                _ => {}
            }
        }

        save_sync_ids(url, remote_ids)?;
        Ok(())
    }
}

use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Result;
use tokio::{sync::RwLock, task::spawn_blocking};

use crate::{
    constants::{SC_ARTWORK_URL, SC_IDENTIFIER, SC_SOURCE_URL},
    database::sync::{get_previous_ids, save_sync_ids},
    models::SyncMode,
    utils::metadata::read_custom_field,
};

#[derive(Default, Clone)]
pub struct TrackData {
    pub path: PathBuf,
    pub artist: String,
    pub title: String,
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
        artist: String,
        title: String,
        artwork_url: Option<String>,
        source_url: Option<String>,
    ) {
        self.tracks.insert(
            id,
            TrackData {
                path,
                artist,
                title,
                artwork_url,
                source_url,
            },
        );
    }

    pub fn remove_track(&mut self, id: i64) {
        self.tracks.remove(&id);
    }

    pub async fn run_background_indexing(storage: Arc<RwLock<Self>>) {
        let root = {
            let s = storage.read().await;
            PathBuf::from(&s.base_path)
        };

        let result = spawn_blocking(move || {
            let mut new_tracks = HashMap::new();
            let mut seen_ids = HashSet::new();
            let mut stack = vec![root];

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

                    if let Some((p_str, id)) = path.to_str().and_then(|p| {
                        read_custom_field(p, SC_IDENTIFIER)
                            .and_then(|s| s.parse::<i64>().ok())
                            .map(|id| (p, id))
                    }) {
                        seen_ids.insert(id);

                        let tag = id3::Tag::read_from_path(&path).ok();
                        let (artist, title) = if let Some(t) = tag {
                            use id3::TagLike;
                            (
                                t.artist().unwrap_or("Unknown").to_string(),
                                t.title().unwrap_or("Unknown").to_string(),
                            )
                        } else {
                            let clean_name = path
                                .file_stem()
                                .map(|s| s.to_string_lossy())
                                .unwrap_or_default()
                                .replace('_', " ");

                            if clean_name.contains(" - ") {
                                let parts: Vec<&str> = clean_name.split(" - ").collect();
                                (parts[0].trim().to_string(), parts[1].trim().to_string())
                            } else {
                                ("Unknown".to_string(), clean_name.to_string())
                            }
                        };

                        let artwork_url = read_custom_field(p_str, SC_ARTWORK_URL);
                        let source_url = read_custom_field(p_str, SC_SOURCE_URL);

                        new_tracks.insert(
                            id,
                            TrackData {
                                path,
                                artist,
                                title,
                                artwork_url,
                                source_url,
                            },
                        );
                    }
                }
            }
            (new_tracks, seen_ids)
        })
        .await;

        if let Ok((new_tracks, seen_ids)) = result {
            let mut s = storage.write().await;
            for (id, data) in new_tracks {
                s.tracks.insert(id, data);
            }
            s.tracks.retain(|id, _| seen_ids.contains(id));
            tracing::info!("Indexing complete. Found {} tracks.", s.tracks.len());
        }
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

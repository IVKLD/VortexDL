use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::UNIX_EPOCH,
};

use anyhow::Result;
use id3::TagLike;
use tokio::{sync::RwLock, task::spawn_blocking};

use crate::{
    constants::{SC_ARTWORK_URL, SC_IDENTIFIER, SC_POSITION, SC_SOURCE_URL},
    database::{get_previous_ids, save_sync_ids},
    types::SyncMode,
    utils::metadata::read_custom_field,
};

#[derive(Default, Clone)]
pub struct TrackData {
    pub path: PathBuf,
    pub artist: String,
    pub title: String,
    pub artwork_url: Option<String>,
    pub source_url: Option<String>,
    pub position: Option<u32>,
    pub created_at: u64,
    pub size: u64,
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
    pub fn update_track(&mut self, id: i64, data: TrackData) {
        self.tracks.insert(id, data);
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

                    let Some(p_str) = path.to_str() else { continue };
                    let Some(id_str) = read_custom_field(p_str, SC_IDENTIFIER) else {
                        continue;
                    };
                    let Ok(id) = id_str.parse::<i64>() else {
                        continue;
                    };

                    let (artist, title) = id3::Tag::read_from_path(&path)
                        .ok()
                        .map(|t| {
                            (
                                t.artist().unwrap_or("Unknown").to_string(),
                                t.title().unwrap_or("Unknown").to_string(),
                            )
                        })
                        .unwrap_or_else(|| Self::parse_filename_fallback(&path));

                    let artwork_url = read_custom_field(p_str, SC_ARTWORK_URL);
                    let source_url = read_custom_field(p_str, SC_SOURCE_URL);
                    let position =
                        read_custom_field(p_str, SC_POSITION).and_then(|s| s.parse().ok());

                    let (created_at, size) = entry
                        .metadata()
                        .ok()
                        .map(|m| {
                            (
                                m.created()
                                    .ok()
                                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                                    .map(|d| d.as_secs())
                                    .unwrap_or(0),
                                m.len(),
                            )
                        })
                        .unwrap_or((0, 0));

                    new_tracks.insert(
                        id,
                        TrackData {
                            path,
                            artist,
                            title,
                            artwork_url,
                            source_url,
                            position,
                            created_at,
                            size,
                        },
                    );
                }
            }
            new_tracks
        })
        .await;

        if let Ok(new_tracks) = result {
            let mut s = storage.write().await;
            let count = new_tracks.len();

            s.tracks = new_tracks;

            tracing::info!("Indexing complete. Found {} tracks.", count);
        }
    }

    fn parse_filename_fallback(path: &Path) -> (String, String) {
        let clean_name = path
            .file_stem()
            .map(|s| s.to_string_lossy())
            .unwrap_or_default()
            .replace('_', " ");

        if let Some((artist, title)) = clean_name.split_once(" - ") {
            (artist.trim().to_string(), title.trim().to_string())
        } else {
            ("Unknown".to_string(), clean_name)
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
            tokio::fs::create_dir_all(&archive_path).await?;
        }

        for data in to_remove {
            if !data.path.exists() {
                continue;
            }

            match mode {
                SyncMode::Full => tokio::fs::remove_file(&data.path).await?,
                SyncMode::Archive => {
                    if let Some(name) = data.path.file_name() {
                        tokio::fs::rename(&data.path, archive_path.join(name)).await?;
                    }
                }
                _ => {}
            }
        }

        save_sync_ids(url, remote_ids)?;
        Ok(())
    }
}

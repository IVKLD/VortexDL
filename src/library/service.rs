use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::Arc,
    time::SystemTime,
};

use anyhow::Result;
use tokio::sync::RwLock;
use url::Url;

use crate::{
    domain::{AudioFormat, SourcePlatform, TrackMetadata},
    infra::{
        db::{CachedTrack, DeletedTrackRecord, StatsRepo, TrackCacheRepo},
        system_time_to_secs,
    },
    library::{
        Library, LibraryStats,
        metadata::{self, SaveTrackArgs},
        scanner::scan_directory,
        types::LibraryTrack,
    },
    settings::SettingsManager,
};

pub struct RegisterDownloadedTrackArgs {
    pub id: i64,
    pub path: PathBuf,
    pub title: String,
    pub artist: String,
    pub artwork_url: Option<Url>,
    pub source_url: Option<Url>,
    pub artwork_data: Option<Vec<u8>>,
    pub size: u64,
}

#[derive(Clone)]
pub struct LibraryService {
    inner: Arc<RwLock<Library>>,
    settings: SettingsManager,
}

impl LibraryService {
    pub fn new(inner: Arc<RwLock<Library>>, settings: SettingsManager) -> Self {
        Self { inner, settings }
    }

    pub fn inner(&self) -> &Arc<RwLock<Library>> {
        &self.inner
    }

    pub async fn reindex(&self) -> usize {
        let output_dir = self.settings.read().await.downloads.output_path.clone();
        let tracks = scan_directory(&output_dir).await;
        let count = tracks.len();
        self.inner.write().await.update_tracks(tracks);
        count
    }

    pub async fn get_track(&self, id: i64) -> Option<LibraryTrack> {
        self.inner.read().await.get_track(id).cloned()
    }

    pub async fn is_track_downloaded(&self, id: i64) -> bool {
        self.inner.read().await.is_track_downloaded(id)
    }

    pub async fn active_tracks(&self) -> Vec<LibraryTrack> {
        self.inner
            .read()
            .await
            .values()
            .filter(|t| !t.is_archived)
            .cloned()
            .collect()
    }

    pub async fn list_tracks(
        &self,
        sort: Option<&str>,
        order: Option<&str>,
        limit: Option<usize>,
    ) -> Vec<(i64, LibraryTrack)> {
        let storage = self.inner.read().await;

        let mut tracks: Vec<(i64, LibraryTrack)> = storage
            .iter()
            .map(|(&id, track)| (id, track.clone()))
            .collect();

        let sort_by = sort.unwrap_or("date");
        let order_by = order.unwrap_or("desc");

        tracks.sort_by(|(id_a, a), (id_b, b)| {
            let cmp = match sort_by {
                "name" => a
                    .metadata
                    .title
                    .to_lowercase()
                    .cmp(&b.metadata.title.to_lowercase()),
                _ => a.created_at.cmp(&b.created_at),
            };

            let cmp = cmp.then_with(|| id_a.cmp(id_b));

            if order_by == "desc" {
                cmp.reverse()
            } else {
                cmp
            }
        });

        if let Some(limit) = limit {
            tracks.truncate(limit);
        }

        tracks
    }

    pub async fn get_stats(&self) -> LibraryStats {
        let storage = self.inner.read().await;
        let total_tracks = storage.values().filter(|t| !t.is_archived).count();
        let total_size: u64 = storage
            .values()
            .filter(|t| !t.is_archived)
            .map(|t| t.size)
            .sum();

        let persistent = StatsRepo::get().unwrap_or_default();

        let total_downloaded = persistent
            .total_downloaded
            .max(total_tracks as u64 + persistent.total_deleted);
        let total_downloaded_bytes = persistent
            .total_downloaded_bytes
            .max(total_size + persistent.total_deleted_bytes);

        LibraryStats {
            total_tracks,
            total_size,
            total_downloaded,
            total_downloaded_bytes,
            total_deleted: persistent.total_deleted,
            total_deleted_bytes: persistent.total_deleted_bytes,
            recent_deleted: persistent.deleted_history,
        }
    }

    pub async fn clear_deleted_stats(&self) -> Result<()> {
        tokio::task::spawn_blocking(StatsRepo::clear_deleted_history).await??;
        Ok(())
    }

    pub async fn remove_tracks(&self, ids: Vec<i64>) -> Result<Vec<LibraryTrack>> {
        let removed = self.inner.write().await.remove_tracks(&ids);
        if removed.is_empty() {
            return Ok(removed);
        }

        let mut to_remove_cache = HashSet::new();
        let now = system_time_to_secs(SystemTime::now());
        let mut audit_records = Vec::with_capacity(removed.len());

        for track in &removed {
            if track.path.exists() {
                let _ = tokio::fs::remove_file(&track.path).await;
            }
            to_remove_cache.insert(track.path.to_string_lossy().into_owned());

            let format = AudioFormat::from_path(&track.path).to_string();
            audit_records.push(DeletedTrackRecord {
                id: track.metadata.id,
                title: track.metadata.title.clone(),
                artist: track.metadata.artist.clone(),
                format,
                size: track.size,
                deleted_at: now,
            });
        }

        tokio::task::spawn_blocking(move || -> Result<()> {
            TrackCacheRepo::update_batch(&HashMap::new(), &to_remove_cache)?;
            StatsRepo::record_deleted_tracks(&audit_records)?;
            Ok(())
        })
        .await??;

        Ok(removed)
    }

    pub async fn register_downloaded_track(
        &self,
        args: RegisterDownloadedTrackArgs,
    ) -> Result<TrackMetadata> {
        let now = system_time_to_secs(SystemTime::now());
        let mtime = tokio::fs::metadata(&args.path)
            .await
            .ok()
            .and_then(|m| m.modified().ok())
            .map(system_time_to_secs)
            .unwrap_or(now);

        let platform =
            SourcePlatform::detect_from_optional_url(args.source_url.as_ref().map(|u| u.as_str()))
                .as_str()
                .to_string();

        let metadata = TrackMetadata {
            id: args.id,
            title: args.title.clone(),
            artist: args.artist.clone(),
            artwork_url: args.artwork_url.as_ref().map(|u| u.to_string()),
            source_url: args.source_url.as_ref().map(|u| u.to_string()),
            platform,
        };

        let path_str = args.path.to_string_lossy().into_owned();
        let file_path = args.path.clone();
        let track_id_str = args.id.to_string();
        let track_title = args.title;
        let track_artist = args.artist;
        let artwork_url = args.artwork_url;
        let source_url = args.source_url;
        let artwork_data = args.artwork_data;
        let metadata_clone = metadata.clone();
        let size = args.size;

        tokio::task::spawn_blocking(move || -> Result<()> {
            metadata::save_track_info(SaveTrackArgs {
                path: &file_path,
                sc_id: &track_id_str,
                title: &track_title,
                artist: &track_artist,
                artwork_url: artwork_url.as_ref(),
                source_url: source_url.as_ref(),
                artwork_data,
            })?;

            let mut to_update = HashMap::new();
            to_update.insert(
                path_str,
                CachedTrack {
                    metadata: metadata_clone,
                    created_at: now,
                    size,
                    mtime,
                },
            );
            TrackCacheRepo::update_batch(&to_update, &HashSet::new())?;
            StatsRepo::record_download(size)?;
            Ok(())
        })
        .await??;

        self.inner.write().await.update_track(
            args.id,
            LibraryTrack {
                path: args.path,
                metadata: metadata.clone(),
                created_at: now,
                size: args.size,
                is_archived: false,
            },
        );

        Ok(metadata)
    }
}

use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

use crate::{
    database,
    database::cache::CachedMusicTrack,
    settings::UserSettings,
    webdav::{error::SyncError, strategy::ISyncStrategy},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct DataSnapshot {
    pub settings: UserSettings,
    pub synced_ids: HashMap<String, HashSet<i64>>,
    pub cached_tracks: HashMap<String, CachedMusicTrack>,
}

impl DataSnapshot {
    #[instrument]
    pub fn from_database() -> Result<Self, SyncError> {
        let settings = database::get_settings()?;
        let synced_ids = database::get_all_sync_ids()?;
        let cached_tracks = database::cache::get_cached_music_tracks()?;
        Ok(Self {
            settings,
            synced_ids,
            cached_tracks,
        })
    }

    #[instrument(skip(self))]
    pub fn apply_to_database(self) -> Result<(), SyncError> {
        database::update_settings(&self.settings)?;
        database::restore_all_sync_ids(&self.synced_ids)?;
        database::cache::update_cached_tracks_batch(&self.cached_tracks, &HashSet::new())?;
        Ok(())
    }
}

pub struct SyncService<S: ISyncStrategy> {
    strategy: S,
}

impl<S: ISyncStrategy> SyncService<S> {
    pub fn new(strategy: S) -> Self {
        Self { strategy }
    }

    #[instrument(skip(self))]
    pub async fn export(&self) -> Result<(), SyncError> {
        let tmp = self.temp_path()?;
        let result = self.do_export(&tmp).await;
        let _ = std::fs::remove_file(&tmp);
        result
    }

    #[instrument(skip(self))]
    pub async fn import(&self) -> Result<(), SyncError> {
        let tmp = self.temp_path()?;
        let result = self.do_import(&tmp).await;
        let _ = std::fs::remove_file(&tmp);
        result
    }

    #[instrument(skip(self))]
    async fn do_export(&self, tmp: &Path) -> Result<(), SyncError> {
        let snapshot = DataSnapshot::from_database()?;

        let json = serde_json::to_string_pretty(&snapshot)?;
        tokio::fs::write(tmp, json.as_bytes()).await?;
        debug!(bytes = json.len(), "snapshot written");

        self.strategy.upload(tmp).await?;
        Ok(())
    }

    #[instrument(skip(self))]
    async fn do_import(&self, tmp: &Path) -> Result<(), SyncError> {
        self.strategy.download(tmp).await?;

        let bytes = tokio::fs::read(tmp).await?;
        debug!(bytes = bytes.len(), "snapshot received");

        let snapshot: DataSnapshot = serde_json::from_slice(&bytes)?;
        snapshot.apply_to_database()?;
        Ok(())
    }

    fn temp_path(&self) -> Result<PathBuf, SyncError> {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);

        let mut path = std::env::temp_dir();
        path.push(format!("vortex_sync_{ts}.json"));
        Ok(path)
    }
}

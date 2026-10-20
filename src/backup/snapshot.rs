use std::{collections::HashSet, path::Path};

use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{backup::error::BackupError, infra::db, settings::UserSettings};

const SNAPSHOT_VERSION: u32 = 1;

fn default_snapshot_version() -> u32 {
    SNAPSHOT_VERSION
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupSnapshot {
    #[serde(default = "default_snapshot_version")]
    pub version: u32,
    pub settings: UserSettings,
    #[serde(default, alias = "synced_ids")]
    pub synced_ids: HashSet<i64>,
}

impl BackupSnapshot {
    #[instrument]
    pub fn from_database() -> Result<Self, BackupError> {
        let mut settings = db::get_settings()?;
        settings.webdav.password.clear();

        let cached = db::get_cached_tracks()?;
        let synced_ids = cached
            .into_iter()
            .filter(|(path, _)| Path::new(path).exists())
            .map(|(_, track)| track.metadata.id)
            .collect();

        Ok(Self {
            version: SNAPSHOT_VERSION,
            settings,
            synced_ids,
        })
    }

    #[instrument(skip(self))]
    pub fn apply_to_database(mut self) -> Result<(), BackupError> {
        if self.settings.webdav.password.is_empty() {
            let current = db::get_settings()?;
            self.settings.webdav.password = current.webdav.password;
        }

        db::update_settings(&self.settings)?;
        Ok(())
    }
}

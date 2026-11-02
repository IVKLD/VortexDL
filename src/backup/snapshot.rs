use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::settings::UserSettings;

pub const SNAPSHOT_VERSION: u32 = 1;

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

use anyhow::Result;
use redb::TableDefinition;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::infra::db::{read_value, write_value};

const STATS_TABLE: TableDefinition<u64, &str> = TableDefinition::new("persistent_stats");

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeletedTrackRecord {
    pub id: i64,
    pub title: String,
    pub artist: String,
    pub format: String,
    pub size: u64,
    pub deleted_at: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PersistentStats {
    pub total_downloaded: u64,
    pub total_downloaded_bytes: u64,
    pub total_deleted: u64,
    pub total_deleted_bytes: u64,
    pub deleted_history: Vec<DeletedTrackRecord>,
}

pub struct StatsRepo;

impl StatsRepo {
    pub fn get() -> Result<PersistentStats> {
        Ok(read_value(STATS_TABLE)?.unwrap_or_default())
    }

    pub fn save(stats: &PersistentStats) -> Result<()> {
        write_value(STATS_TABLE, stats)
    }

    pub fn record_download(size: u64) -> Result<()> {
        let mut stats = Self::get().unwrap_or_default();
        stats.total_downloaded += 1;
        stats.total_downloaded_bytes += size;
        Self::save(&stats)
    }

    pub fn record_deleted_tracks(tracks: &[DeletedTrackRecord]) -> Result<()> {
        if tracks.is_empty() {
            return Ok(());
        }
        let mut stats = Self::get().unwrap_or_default();
        for track in tracks {
            stats.total_deleted += 1;
            stats.total_deleted_bytes += track.size;
            stats.deleted_history.insert(0, track.clone());
        }
        if stats.deleted_history.len() > 100 {
            stats.deleted_history.truncate(100);
        }
        Self::save(&stats)
    }

    pub fn clear_deleted_history() -> Result<()> {
        let mut stats = Self::get().unwrap_or_default();
        stats.deleted_history.clear();
        Self::save(&stats)
    }
}

use anyhow::Result;
use redb::{ReadableDatabase, TableDefinition};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::infra::db::get_db;

const STATS_TABLE: TableDefinition<u64, &str> = TableDefinition::new("persistent_stats");
const STATS_ID: u64 = 0;

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
        let db = get_db();
        let read_txn = db.begin_read()?;

        let table = match read_txn.open_table(STATS_TABLE) {
            Ok(t) => t,
            Err(_) => return Ok(PersistentStats::default()),
        };

        let Some(data) = table.get(STATS_ID)? else {
            return Ok(PersistentStats::default());
        };

        Ok(serde_json::from_str(data.value()).unwrap_or_default())
    }

    pub fn save(stats: &PersistentStats) -> Result<()> {
        let db = get_db();
        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(STATS_TABLE)?;
            let json = serde_json::to_string(stats)?;
            table.insert(STATS_ID, json.as_str())?;
        }
        write_txn.commit()?;
        Ok(())
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

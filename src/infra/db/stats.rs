use anyhow::Result;
use redb::{ReadableDatabase, TableDefinition};
use serde::{Deserialize, Serialize};

use super::get_db;

const STATS_TABLE: TableDefinition<u64, &str> = TableDefinition::new("stats_v1");
const STATS_KEY: u64 = 0;
const MAX_DELETED_HISTORY: usize = 100;

#[derive(Serialize, Deserialize, Clone, Debug, Default, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeletedTrackRecord {
    pub id: i64,
    pub title: String,
    pub artist: String,
    pub format: String,
    pub size: u64,
    pub deleted_at: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PersistentStats {
    pub total_downloaded: u64,
    pub total_downloaded_bytes: u64,
    pub total_deleted: u64,
    pub total_deleted_bytes: u64,
    pub deleted_history: Vec<DeletedTrackRecord>,
}

pub fn get_stats() -> Result<PersistentStats> {
    let db = get_db();
    let read_txn = db.begin_read()?;

    let table = match read_txn.open_table(STATS_TABLE) {
        Ok(t) => t,
        Err(_) => return Ok(PersistentStats::default()),
    };

    let Some(data) = table.get(STATS_KEY)? else {
        return Ok(PersistentStats::default());
    };

    match serde_json::from_str(data.value()) {
        Ok(stats) => Ok(stats),
        Err(err) => {
            tracing::error!("Failed to parse stats from database: {err}. Falling back to default.");
            Ok(PersistentStats::default())
        }
    }
}

pub fn save_stats(stats: &PersistentStats) -> Result<()> {
    let db = get_db();
    let write_txn = db.begin_write()?;
    {
        let mut table = write_txn.open_table(STATS_TABLE)?;
        let json = serde_json::to_string(stats)?;
        table.insert(STATS_KEY, json.as_str())?;
    }
    write_txn.commit()?;
    Ok(())
}

pub fn record_download(size: u64) -> Result<()> {
    let mut stats = get_stats().unwrap_or_default();
    stats.total_downloaded = stats.total_downloaded.saturating_add(1);
    stats.total_downloaded_bytes = stats.total_downloaded_bytes.saturating_add(size);
    save_stats(&stats)
}

pub fn record_deleted_tracks(tracks: &[DeletedTrackRecord]) -> Result<()> {
    if tracks.is_empty() {
        return Ok(());
    }

    let mut stats = get_stats().unwrap_or_default();
    let count = tracks.len() as u64;
    let size_sum: u64 = tracks.iter().map(|t| t.size).sum();

    stats.total_deleted = stats.total_deleted.saturating_add(count);
    stats.total_deleted_bytes = stats.total_deleted_bytes.saturating_add(size_sum);

    let mut new_history = tracks.to_vec();
    new_history.extend(stats.deleted_history);
    new_history.truncate(MAX_DELETED_HISTORY);
    stats.deleted_history = new_history;

    save_stats(&stats)
}

pub fn clear_deleted_history() -> Result<()> {
    let mut stats = get_stats().unwrap_or_default();
    stats.deleted_history.clear();
    save_stats(&stats)
}

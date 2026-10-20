use std::collections::{HashMap, HashSet};

use anyhow::Result;
use redb::{ReadableDatabase, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};

use super::get_db;
use crate::domain::TrackMetadata;

const CACHE_TABLE: TableDefinition<&str, &str> = TableDefinition::new("track_cache");

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CachedTrack {
    #[serde(flatten)]
    pub metadata: TrackMetadata,
    pub created_at: u64,
    pub size: u64,
    pub mtime: u64,
}

pub fn get_cached_tracks() -> Result<HashMap<String, CachedTrack>> {
    let db = get_db();
    let read_txn = db.begin_read()?;

    let mut map = HashMap::new();
    let table = match read_txn.open_table(CACHE_TABLE) {
        Ok(t) => t,
        Err(_) => return Ok(map),
    };

    for (key, value) in table.iter()?.flatten() {
        if let Ok(cached) = serde_json::from_str::<CachedTrack>(value.value()) {
            map.insert(key.value().to_string(), cached);
        }
    }

    Ok(map)
}

pub fn update_cached_tracks_batch(
    to_update: &HashMap<String, CachedTrack>,
    to_remove: &HashSet<String>,
) -> Result<()> {
    if to_update.is_empty() && to_remove.is_empty() {
        return Ok(());
    }
    let db = get_db();
    let write_txn = db.begin_write()?;
    {
        let mut table = write_txn.open_table(CACHE_TABLE)?;
        for path in to_remove {
            table.remove(path.as_str())?;
        }
        for (path, cached) in to_update {
            let json = serde_json::to_string(cached)?;
            table.insert(path.as_str(), json.as_str())?;
        }
    }
    write_txn.commit()?;
    Ok(())
}

pub fn remove_cached_track(path: &str) -> Result<()> {
    let mut to_remove = HashSet::with_capacity(1);
    to_remove.insert(path.to_string());
    update_cached_tracks_batch(&HashMap::new(), &to_remove)
}

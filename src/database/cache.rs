use std::collections::HashMap;

use anyhow::Result;
use redb::{ReadableDatabase, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};

use crate::database::get_db;

const CACHE_TABLE: TableDefinition<&str, &str> = TableDefinition::new("track_cache");

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CachedMusicTrack {
    pub id: i64,
    pub artist: String,
    pub title: String,
    pub artwork_url: Option<String>,
    pub source_url: Option<String>,
    pub position: Option<u32>,
    pub created_at: u64,
    pub size: u64,
    pub mtime: u64,
}

pub fn get_cached_music_tracks() -> Result<HashMap<String, CachedMusicTrack>> {
    let db = get_db();
    let read_txn = db.begin_read()?;

    let mut map = HashMap::new();
    let table = match read_txn.open_table(CACHE_TABLE) {
        Ok(t) => t,
        Err(_) => return Ok(map),
    };

    for (key, value) in table.iter()?.flatten() {
        if let Ok(cached) = serde_json::from_str::<CachedMusicTrack>(value.value()) {
            map.insert(key.value().to_string(), cached);
        }
    }

    Ok(map)
}

pub fn save_cached_music_tracks(tracks: &HashMap<String, CachedMusicTrack>) -> Result<()> {
    let db = get_db();
    let write_txn = db.begin_write()?;
    {
        let _ = write_txn.delete_table(CACHE_TABLE);
        let mut table = write_txn.open_table(CACHE_TABLE)?;
        for (path, cached) in tracks {
            let json = serde_json::to_string(cached)?;
            table.insert(path.as_str(), json.as_str())?;
        }
    }
    write_txn.commit()?;
    Ok(())
}

use std::collections::{HashMap, HashSet};

use anyhow::Error;
use redb::{ReadableDatabase, ReadableTable, TableDefinition};

use crate::database::get_db;

const SYNC_TABLE: TableDefinition<&str, &str> = TableDefinition::new("sync_log");

pub fn get_previous_ids(url: &str) -> Result<HashSet<i64>, Error> {
    let db = get_db();
    let read_txn = db.begin_read()?;

    let ids = read_txn
        .open_table(SYNC_TABLE)
        .ok()
        .and_then(|table| table.get(url).ok().flatten())
        .and_then(|data| serde_json::from_str(data.value()).ok())
        .unwrap_or_default();

    Ok(ids)
}

pub fn save_sync_ids(url: &str, ids: &HashSet<i64>) -> Result<(), Error> {
    let db = get_db();
    let write_txn = db.begin_write()?;
    {
        let mut table = write_txn.open_table(SYNC_TABLE)?;
        let json = serde_json::to_string(ids)?;
        table.insert(url, json.as_str())?;
    }
    write_txn.commit()?;
    Ok(())
}

pub fn get_all_sync_ids() -> Result<HashMap<String, HashSet<i64>>, Error> {
    let db = get_db();
    let read_txn = db.begin_read()?;

    let mut map = HashMap::new();
    let table = match read_txn.open_table(SYNC_TABLE) {
        Ok(t) => t,
        Err(_) => return Ok(map),
    };

    for result in table.iter()? {
        let (key, value) = result?;
        if let Ok(ids) = serde_json::from_str::<HashSet<i64>>(value.value()) {
            map.insert(key.value().to_string(), ids);
        }
    }

    Ok(map)
}

pub fn restore_all_sync_ids(data: &HashMap<String, HashSet<i64>>) -> Result<(), Error> {
    let db = get_db();
    let write_txn = db.begin_write()?;
    {
        let mut table = write_txn.open_table(SYNC_TABLE)?;
        for (url, ids) in data {
            let json = serde_json::to_string(ids)?;
            table.insert(url.as_str(), json.as_str())?;
        }
    }
    write_txn.commit()?;
    Ok(())
}

use std::collections::HashSet;

use anyhow::Error;
use redb::{ReadableDatabase, TableDefinition};

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


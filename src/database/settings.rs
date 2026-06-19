use anyhow::Error;
use redb::{ReadableDatabase, TableDefinition};

use crate::{database::get_db, settings::UserSettings};

const SETTINGS_TABLE: TableDefinition<u64, &str> = TableDefinition::new("settings");
const SETTINGS_ID: u64 = 0;

pub fn get_settings() -> Result<UserSettings, Error> {
    let db = get_db();
    let read_txn = db.begin_read()?;

    let settings = read_txn
        .open_table(SETTINGS_TABLE)
        .ok()
        .and_then(|table| table.get(SETTINGS_ID).ok().flatten())
        .and_then(|data| serde_json::from_str(data.value()).ok())
        .unwrap_or_default();

    Ok(settings)
}

pub fn update_settings(settings: &UserSettings) -> Result<(), Error> {
    let db = get_db();
    let write_txn = db.begin_write()?;
    {
        let mut table = write_txn.open_table(SETTINGS_TABLE)?;
        let json = serde_json::to_string(settings)?;
        table.insert(SETTINGS_ID, json.as_str())?;
    }
    write_txn.commit()?;

    Ok(())
}

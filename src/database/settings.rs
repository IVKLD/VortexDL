use anyhow::Error;
use redb::{ReadableDatabase, TableDefinition};
use serde::{Deserialize, Serialize};

use crate::database::get_db;

const TABLE: TableDefinition<u64, &str> = TableDefinition::new("settings");
const SETTINGS_ID: u64 = 0;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserSettings {
    pub sc_user_url: String,
}

pub fn get_settings() -> Result<Option<UserSettings>, Error> {
    let db = get_db();
    let read_txn = db.begin_read()?;

    let table = match read_txn.open_table(TABLE) {
        Ok(t) => t,
        Err(_) => return Ok(None),
    };

    if let Some(data) = table.get(SETTINGS_ID)? {
        let settings: UserSettings = serde_json::from_str(data.value())?;
        return Ok(Some(settings));
    }

    Ok(None)
}

pub fn update_settings(settings: &UserSettings) -> Result<(), Error> {
    let db = get_db();
    let write_txn = db.begin_write()?;
    {
        let mut table = write_txn.open_table(TABLE)?;
        let json = serde_json::to_string(settings)?;
        table.insert(SETTINGS_ID, json.as_str())?;
    }
    write_txn.commit()?;

    Ok(())
}

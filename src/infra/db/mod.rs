use std::{
    fs,
    path::PathBuf,
    sync::{LazyLock, OnceLock},
};

use anyhow::{Error, anyhow};
use redb::{Database, ReadableDatabase};

pub mod cache;
pub mod settings;
pub mod stats;

pub use cache::{CachedTrack, TrackCacheRepo};
pub use settings::SettingsRepo;
pub use stats::{DeletedTrackRecord, PersistentStats, StatsRepo};

use crate::infra::{APP_DIR, DB_NAME};

static DB_FILE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(APP_DIR)
        .join(DB_NAME)
});

static DB: OnceLock<Database> = OnceLock::new();

pub fn get_db() -> &'static Database {
    DB.get()
        .expect("Database not initialized. Call db::init() first.")
}

pub fn init() -> Result<(), Error> {
    if let Some(parent) = DB_FILE_PATH.parent() {
        fs::create_dir_all(parent)?;
    }

    let db = Database::create(&*DB_FILE_PATH)?;
    DB.set(db)
        .map_err(|_| anyhow!("Database already initialized"))?;

    Ok(())
}

pub fn get_singleton<T: serde::de::DeserializeOwned>(
    table_def: redb::TableDefinition<u64, &str>,
) -> Result<Option<T>, Error> {
    let db = get_db();
    let read_txn = db.begin_read()?;
    let table = match read_txn.open_table(table_def) {
        Ok(t) => t,
        Err(_) => return Ok(None),
    };
    let Some(data) = table.get(0)? else {
        return Ok(None);
    };
    Ok(serde_json::from_str(data.value()).ok())
}

pub fn set_singleton<T: serde::Serialize>(
    table_def: redb::TableDefinition<u64, &str>,
    value: &T,
) -> Result<(), Error> {
    let db = get_db();
    let write_txn = db.begin_write()?;
    {
        let mut table = write_txn.open_table(table_def)?;
        let json = serde_json::to_string(value)?;
        table.insert(0, json.as_str())?;
    }
    write_txn.commit()?;
    Ok(())
}

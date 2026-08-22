use std::{
    fs,
    path::PathBuf,
    sync::{LazyLock, OnceLock},
};

use anyhow::{Error, anyhow};
use redb::Database;

pub mod cache;
pub mod settings;
pub mod sync_log;

pub use settings::{get_settings, update_settings};
pub use sync_log::{get_all_sync_ids, get_previous_ids, restore_all_sync_ids, save_sync_ids};

use crate::constants::{APP_DIR, DB_NAME};

static DB_FILE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    dirs::config_dir()
        .expect("Could not find config directory")
        .join(APP_DIR)
        .join(DB_NAME)
});

static DB: OnceLock<Database> = OnceLock::new();

pub fn get_db() -> &'static Database {
    DB.get()
        .expect("Database not initialized. Call database::init() first.")
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

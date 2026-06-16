use std::{
    fs,
    path::PathBuf,
    sync::{Arc, LazyLock, OnceLock},
};

use anyhow::{Error, anyhow};
use redb::Database;

pub mod settings;
pub mod sync;

pub use settings::{get_settings, update_settings};
pub use sync::{get_previous_ids, save_sync_ids};

use crate::constants::{APP_DIR, DB_NAME};

static DB_FILE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    dirs::config_dir()
        .expect("Could not find config directory")
        .join(APP_DIR)
        .join(DB_NAME)
});

static DB: OnceLock<Arc<Database>> = OnceLock::new();

pub fn get_db() -> Arc<Database> {
    DB.get()
        .expect("Database not initialized. Call database::init() first.")
        .clone()
}

pub fn init() -> Result<(), Error> {
    if let Some(parent) = DB_FILE_PATH.parent() {
        fs::create_dir_all(parent)?;
    }

    let db = Database::create(&*DB_FILE_PATH)?;
    DB.set(Arc::new(db))
        .map_err(|_| anyhow!("Database already initialized"))?;

    Ok(())
}

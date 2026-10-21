use std::{
    fs,
    path::PathBuf,
    sync::{LazyLock, OnceLock},
};

use anyhow::{Error, anyhow};
use redb::Database;

pub mod cache;
pub mod settings;
pub mod stats;

pub use cache::{CachedTrack, get_cached_tracks, remove_cached_track, update_cached_tracks_batch};
pub use settings::{get_settings, update_settings};
pub use stats::{
    DeletedTrackRecord, PersistentStats, clear_deleted_history, get_stats, record_deleted_tracks,
    record_download,
};

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

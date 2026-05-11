use anyhow::Error;
use redb::{ReadableDatabase, TableDefinition};
use serde::{Deserialize, Serialize};

use crate::database::get_db;

const TABLE: TableDefinition<u64, &str> = TableDefinition::new("settings");
const SETTINGS_ID: u64 = 0;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SoundcloudSettings {
    pub profile_url: String,
    pub sync_interval: u32,
    pub auto_sync: bool,
    pub cached_client_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DownloadSettings {
    pub output_path: String,
    pub max_concurrent: u32,
    pub naming_template: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserSettings {
    pub soundcloud: SoundcloudSettings,
    pub downloads: DownloadSettings,
    pub limit_per_page: u32,
    pub max_retries: u32,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            soundcloud: SoundcloudSettings {
                profile_url: String::new(),
                sync_interval: 60,
                auto_sync: true,
                cached_client_id: None,
            },
            downloads: DownloadSettings {
                output_path: "./downloads".to_string(),
                max_concurrent: 3,
                naming_template: "{artist} - {title}".to_string(),
            },
            limit_per_page: 100,
            max_retries: 5,
        }
    }
}

pub fn get_settings() -> Result<UserSettings, Error> {
    let db = get_db();
    let read_txn = db.begin_read()?;

    let table = match read_txn.open_table(TABLE) {
        Ok(t) => t,
        Err(_) => return Ok(UserSettings::default()),
    };

    let settings = table
        .get(SETTINGS_ID)?
        .and_then(|data| serde_json::from_str(data.value()).ok())
        .unwrap_or_else(UserSettings::default);

    Ok(settings)
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

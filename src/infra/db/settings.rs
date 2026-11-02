use anyhow::Error;
use redb::TableDefinition;

use crate::{
    infra::db::{read_value, write_value},
    settings::UserSettings,
};

const SETTINGS_TABLE: TableDefinition<u64, &str> = TableDefinition::new("settings");

pub struct SettingsRepo;

impl SettingsRepo {
    pub fn get() -> Result<UserSettings, Error> {
        Ok(read_value(SETTINGS_TABLE)?.unwrap_or_default())
    }

    pub fn update(settings: &UserSettings) -> Result<(), Error> {
        write_value(SETTINGS_TABLE, settings)
    }
}

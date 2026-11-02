use anyhow::Error;
use redb::TableDefinition;

use crate::{
    infra::db::{get_singleton, set_singleton},
    settings::UserSettings,
};

const SETTINGS_TABLE: TableDefinition<u64, &str> = TableDefinition::new("settings");

pub struct SettingsRepo;

impl SettingsRepo {
    pub fn get() -> Result<UserSettings, Error> {
        Ok(get_singleton(SETTINGS_TABLE)?.unwrap_or_default())
    }

    pub fn update(settings: &UserSettings) -> Result<(), Error> {
        set_singleton(SETTINGS_TABLE, settings)
    }
}

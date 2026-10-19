use std::sync::Arc;

use anyhow::Result;
use tokio::sync::{RwLock, RwLockReadGuard, watch};

use super::model::UserSettings;
use crate::database::update_settings;

#[derive(Clone)]
pub struct SettingsManager {
    inner: Arc<RwLock<UserSettings>>,
    change_tx: Arc<watch::Sender<()>>,
}

impl SettingsManager {
    pub fn new(initial: UserSettings) -> Self {
        let (change_tx, _) = watch::channel(());
        Self {
            inner: Arc::new(RwLock::new(initial)),
            change_tx: Arc::new(change_tx),
        }
    }

    pub async fn read(&self) -> RwLockReadGuard<'_, UserSettings> {
        self.inner.read().await
    }

    pub fn get_proxy_url_sync(&self) -> Option<String> {
        self.inner
            .try_read()
            .ok()
            .and_then(|s| s.network.get_proxy_url().map(String::from))
    }

    pub async fn update(&self, new_settings: UserSettings) -> Result<()> {
        update_settings(&new_settings)?;
        *self.inner.write().await = new_settings;
        let _ = self.change_tx.send(());
        Ok(())
    }

    pub fn subscribe_changes(&self) -> watch::Receiver<()> {
        self.change_tx.subscribe()
    }
}

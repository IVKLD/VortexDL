use std::path::{Path, PathBuf};

use tracing::{debug, instrument};

use crate::backup::{error::BackupError, snapshot::BackupSnapshot, strategy::BackupStrategy};

pub struct BackupService<S: BackupStrategy> {
    strategy: S,
}

impl<S: BackupStrategy> BackupService<S> {
    pub fn new(strategy: S) -> Self {
        Self { strategy }
    }

    #[instrument(skip(self))]
    pub async fn export(&self) -> Result<(), BackupError> {
        let tmp = self.temp_path()?;
        let result = self.do_export(&tmp).await;
        let _ = std::fs::remove_file(&tmp);
        result
    }

    #[instrument(skip(self))]
    pub async fn import(&self) -> Result<(), BackupError> {
        let tmp = self.temp_path()?;
        let result = self.do_import(&tmp).await;
        let _ = std::fs::remove_file(&tmp);
        result
    }

    #[instrument(skip(self))]
    async fn do_export(&self, tmp: &Path) -> Result<(), BackupError> {
        let snapshot = BackupSnapshot::from_database()?;

        let json = serde_json::to_string_pretty(&snapshot)?;
        tokio::fs::write(tmp, json.as_bytes()).await?;
        debug!(bytes = json.len(), "snapshot written");

        self.strategy.upload(tmp).await?;
        Ok(())
    }

    #[instrument(skip(self))]
    async fn do_import(&self, tmp: &Path) -> Result<(), BackupError> {
        self.strategy.download(tmp).await?;

        let bytes = tokio::fs::read(tmp).await?;
        debug!(bytes = bytes.len(), "snapshot received");

        let snapshot: BackupSnapshot = serde_json::from_slice(&bytes)?;
        snapshot.apply_to_database()?;
        Ok(())
    }

    fn temp_path(&self) -> Result<PathBuf, BackupError> {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);

        let mut path = std::env::temp_dir();
        path.push(format!("vortex_backup_{ts}.json"));
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::settings::UserSettings;

    #[test]
    fn test_snapshot_serialization() {
        let mut settings = UserSettings::default();
        settings.webdav.password = "secret123".to_string();
        settings.webdav.password.clear();

        let mut synced_ids = HashSet::new();
        synced_ids.insert(12345);

        let snapshot = BackupSnapshot {
            version: 1,
            settings,
            synced_ids,
        };

        let json = serde_json::to_string_pretty(&snapshot).unwrap();
        assert!(json.contains("\"version\": 1"));
        assert!(json.contains("\"syncedIds\": [\n    12345\n  ]"));
        assert!(!json.contains("secret123"));
    }
}

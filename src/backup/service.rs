use std::path::Path;

use serde_json::{from_slice, to_string_pretty};
use tracing::{debug, instrument};

use super::{BackupAction, BackupError, BackupSnapshot, strategy::BackupStrategy};

pub struct BackupService<S: BackupStrategy> {
    strategy: S,
}

fn create_temp_backup_file() -> Result<tempfile::NamedTempFile, BackupError> {
    tempfile::Builder::new()
        .prefix("vortex_backup_")
        .suffix(".json")
        .tempfile()
        .map_err(BackupError::FileSystem)
}

impl<S: BackupStrategy> BackupService<S> {
    pub fn new(strategy: S) -> Self {
        Self { strategy }
    }

    pub async fn execute_action(&self, action: BackupAction) -> Result<(), BackupError> {
        match action {
            BackupAction::Export => self.export().await,
            BackupAction::Import => self.import().await,
        }
    }

    #[instrument(skip(self))]
    pub async fn export(&self) -> Result<(), BackupError> {
        let temp_file = create_temp_backup_file()?;
        self.do_export(temp_file.path()).await
    }

    #[instrument(skip(self))]
    pub async fn import(&self) -> Result<(), BackupError> {
        let temp_file = create_temp_backup_file()?;
        self.do_import(temp_file.path()).await
    }

    #[instrument(skip(self))]
    async fn do_export(&self, tmp: &Path) -> Result<(), BackupError> {
        let snapshot = BackupSnapshot::from_database()?;

        let json = to_string_pretty(&snapshot)?;
        tokio::fs::write(tmp, json.as_bytes()).await?;
        debug!("Snapshot written ({} bytes)", json.len());

        self.strategy.upload(tmp).await?;
        Ok(())
    }

    #[instrument(skip(self))]
    async fn do_import(&self, tmp: &Path) -> Result<(), BackupError> {
        self.strategy.download(tmp).await?;

        let bytes = tokio::fs::read(tmp).await?;
        debug!("Snapshot received ({} bytes)", bytes.len());

        let snapshot: BackupSnapshot = from_slice(&bytes)?;
        snapshot.apply_to_database()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::settings::UserSettings;

    #[test]
    fn test_snapshot_serialization() {
        let settings = UserSettings::default();

        let mut synced_ids = HashSet::new();
        synced_ids.insert(12345);

        let snapshot = BackupSnapshot {
            version: 1,
            settings,
            synced_ids,
        };

        let json = to_string_pretty(&snapshot).unwrap();
        assert!(json.contains("\"version\": 1"));
        assert!(json.contains("\"syncedIds\": [\n    12345\n  ]"));
        assert!(!json.contains("secret123"));
    }
}

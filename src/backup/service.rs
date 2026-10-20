use std::path::Path;

use serde_json::{from_slice, to_string_pretty};
use tracing::{debug, instrument};

use super::{BackupAction, BackupBackend, BackupError, BackupSnapshot};

pub struct BackupService<B: BackupBackend> {
    backend: B,
}

fn create_temp_backup_file() -> Result<tempfile::NamedTempFile, BackupError> {
    tempfile::Builder::new()
        .prefix("vortex_backup_")
        .suffix(".json")
        .tempfile()
        .map_err(BackupError::FileSystem)
}

impl<B: BackupBackend> BackupService<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
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

        self.backend.upload(tmp).await?;
        Ok(())
    }

    #[instrument(skip(self))]
    async fn do_import(&self, tmp: &Path) -> Result<(), BackupError> {
        self.backend.download(tmp).await?;

        let bytes = tokio::fs::read(tmp).await?;
        debug!("Read {} bytes from downloaded backup", bytes.len());

        let snapshot: BackupSnapshot = from_slice(&bytes)?;
        snapshot.apply_to_database()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_serialization() {
        let snapshot = BackupSnapshot::default();
        let json = to_string_pretty(&snapshot).unwrap();
        let deserialized: BackupSnapshot = from_slice(json.as_bytes()).unwrap();
        assert_eq!(snapshot.synced_ids, deserialized.synced_ids);
    }
}

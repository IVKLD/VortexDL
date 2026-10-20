use std::path::{Path, PathBuf};

use tracing::{debug, instrument};

use super::super::{backend::BackupBackend, error::BackupError};

pub struct LocalBackend {
    remote_path: PathBuf,
}

impl LocalBackend {
    pub fn new(remote_path: impl Into<PathBuf>) -> Self {
        Self {
            remote_path: remote_path.into(),
        }
    }

    fn validate_path(&self) -> Result<(), BackupError> {
        if self.remote_path.extension().and_then(|s| s.to_str()) != Some("json") {
            return Err(BackupError::OperationNotSupported(
                "Backup file must have a .json extension".to_string(),
            ));
        }
        if self
            .remote_path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            return Err(BackupError::OperationNotSupported(
                "Path traversal is not permitted".to_string(),
            ));
        }
        Ok(())
    }
}

impl BackupBackend for LocalBackend {
    #[instrument(skip(self), fields(remote = %self.remote_path.display()))]
    async fn upload(&self, src: &Path) -> Result<(), BackupError> {
        self.validate_path()?;
        debug!(
            "Local upload: {} -> {}",
            src.display(),
            self.remote_path.display()
        );

        if let Some(parent) = self.remote_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::copy(src, &self.remote_path).await?;
        debug!("Local upload complete");
        Ok(())
    }

    #[instrument(skip(self), fields(remote = %self.remote_path.display()))]
    async fn download(&self, dest: &Path) -> Result<(), BackupError> {
        self.validate_path()?;
        debug!(
            "Local download: {} -> {}",
            self.remote_path.display(),
            dest.display()
        );

        if !self.remote_path.exists() {
            return Err(BackupError::NotFound);
        }

        tokio::fs::copy(&self.remote_path, dest).await?;
        debug!("Local download complete");
        Ok(())
    }
}

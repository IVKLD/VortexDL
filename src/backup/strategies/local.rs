use std::path::{Path, PathBuf};

use tracing::{debug, instrument};

use crate::backup::{error::BackupError, strategy::BackupStrategy};

pub struct LocalStrategy {
    remote_path: PathBuf,
}

impl LocalStrategy {
    pub fn new(remote_path: impl Into<PathBuf>) -> Self {
        Self {
            remote_path: remote_path.into(),
        }
    }
}

impl BackupStrategy for LocalStrategy {
    #[instrument(skip(self), fields(remote = %self.remote_path.display()))]
    async fn upload(&self, src: &Path) -> Result<(), BackupError> {
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

use std::path::Path;

use crate::backup::error::BackupError;

pub trait BackupStrategy: Send + Sync {
    async fn upload(&self, src: &Path) -> Result<(), BackupError>;
    async fn download(&self, dest: &Path) -> Result<(), BackupError>;
}

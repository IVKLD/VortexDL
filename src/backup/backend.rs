use std::path::Path;

use super::error::BackupError;

#[allow(async_fn_in_trait)]
pub trait BackupBackend: Send + Sync {
    async fn upload(&self, src: &Path) -> Result<(), BackupError>;
    async fn download(&self, dest: &Path) -> Result<(), BackupError>;
}

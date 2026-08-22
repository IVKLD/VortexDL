use std::path::Path;

use crate::webdav::error::SyncError;

pub trait ISyncStrategy: Send + Sync {
    async fn upload(&self, src: &Path) -> Result<(), SyncError>;
    async fn download(&self, dest: &Path) -> Result<(), SyncError>;
}

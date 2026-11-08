use std::{io, path::Path, time::UNIX_EPOCH};

use tokio::fs;

/// Centralized file system service for VortexDL.
/// Encapsulates directory management, safe file removal, metadata inspection, and disk I/O.
pub struct FileService;

impl FileService {
    /// Ensure that a directory exists, creating all parent directories if necessary.
    pub async fn ensure_dir(path: &Path) -> io::Result<()> {
        fs::create_dir_all(path).await
    }

    /// Ensure that the parent directory of a file path exists.
    pub async fn ensure_parent_dir(file_path: &Path) -> io::Result<()> {
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        Ok(())
    }

    /// Safely remove a file from disk if it exists, ignoring `NotFound` errors.
    pub async fn remove_file_if_exists(path: &Path) -> io::Result<()> {
        match fs::remove_file(path).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e),
        }
    }

    /// Retrieve the size of a file in bytes.
    pub async fn file_size(path: &Path) -> io::Result<u64> {
        let meta = fs::metadata(path).await?;
        Ok(meta.len())
    }

    /// Retrieve the modification time (mtime) of a file in seconds since UNIX epoch.
    pub async fn mtime_secs(path: &Path) -> Option<u64> {
        let meta = fs::metadata(path).await.ok()?;
        let modified = meta.modified().ok()?;
        modified.duration_since(UNIX_EPOCH).ok().map(|d| d.as_secs())
    }

    /// Copy a file, ensuring the target's parent directory exists.
    pub async fn copy(from: &Path, to: &Path) -> io::Result<u64> {
        Self::ensure_parent_dir(to).await?;
        fs::copy(from, to).await
    }

    /// Synchronous directory creation (for startup/bootstrap phases).
    pub fn ensure_dir_sync(path: &Path) -> io::Result<()> {
        std::fs::create_dir_all(path)
    }

    /// Synchronous parent directory creation.
    pub fn ensure_parent_dir_sync(file_path: &Path) -> io::Result<()> {
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(())
    }
}

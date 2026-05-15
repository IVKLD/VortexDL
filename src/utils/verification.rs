use anyhow::{Result, anyhow};
use tokio::fs;

/// Verifies that the file at the given path exists and has a reasonable size.
/// If expected_size is > 0, it also checks for exact size match.
pub async fn verify_file(path: &str, expected_size: u64) -> Result<()> {
    let final_size = fs::metadata(path).await?.len();

    if final_size < 10_000 || (expected_size > 0 && final_size != expected_size) {
        fs::remove_file(path).await.ok();
        return Err(anyhow!(
            "Integrity check failed: expected {} bytes, got on disk {}",
            expected_size,
            final_size
        ));
    }
    Ok(())
}

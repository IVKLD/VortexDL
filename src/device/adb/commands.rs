use std::collections::HashSet;

use anyhow::{Context, Result};
use tokio::process::Command;

pub async fn ensure_remote_dir(device_id: &str, remote_dir: &str) -> Result<()> {
    let status = Command::new("adb")
        .args(["-s", device_id, "shell", "mkdir", "-p", remote_dir])
        .status()
        .await
        .context("Failed to run adb shell mkdir")?;

    if !status.success() {
        anyhow::bail!("Failed to create remote directory {remote_dir} on {device_id}");
    }

    Ok(())
}

pub async fn list_remote_files(device_id: &str, remote_dir: &str) -> Result<HashSet<String>> {
    let output = Command::new("adb")
        .args(["-s", device_id, "shell", "ls", remote_dir])
        .output()
        .await
        .context("Failed to run adb shell ls")?;

    if !output.status.success() {
        return Ok(HashSet::new());
    }

    let files = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(files)
}

pub async fn run_adb_push(device_id: &str, local_path: &str, remote_path: &str) -> Result<bool> {
    let status = Command::new("adb")
        .args(["-s", device_id, "push", local_path, remote_path])
        .status()
        .await
        .context("adb push spawn failed")?;

    Ok(status.success())
}

pub async fn run_adb_rm(device_id: &str, remote_path: &str) -> Result<bool> {
    let status = Command::new("adb")
        .args(["-s", device_id, "shell", "rm", "-f", remote_path])
        .status()
        .await
        .context("adb rm spawn failed")?;

    Ok(status.success())
}

use std::collections::HashSet;

use anyhow::{Context, Result};
use tokio::process::Command;

async fn adb(device: &str, args: &[&str]) -> Result<String> {
    let output = Command::new("adb")
        .arg("-s")
        .arg(device)
        .args(args)
        .output()
        .await
        .with_context(|| format!("adb {} spawn failed", args.first().unwrap_or(&"")))?;

    if !output.status.success() {
        anyhow::bail!("{}", String::from_utf8_lossy(&output.stderr).trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

pub async fn list_devices() -> Result<HashSet<String>> {
    let output = Command::new("adb")
        .arg("devices")
        .output()
        .await
        .context("Failed to run `adb devices`")?;

    if !output.status.success() {
        anyhow::bail!(
            "adb devices: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .skip(1)
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let id = parts.next()?;
            (parts.next()? == "device").then(|| id.to_string())
        })
        .collect())
}

pub async fn ensure_dir(device: &str, dir: &str) -> Result<()> {
    adb(device, &["shell", "mkdir", "-p", &shell_escape(dir)])
        .await
        .map(|_| ())
}

pub async fn list_files(device: &str, dir: &str) -> Result<HashSet<String>> {
    let escaped = shell_escape(dir);
    let result = adb(device, &["shell", "find", &escaped, "-type", "f"]).await;

    let stdout = match result {
        Ok(s) => s,
        Err(_) => return Ok(HashSet::new()),
    };

    let prefix = format!("{}/", dir.trim_end_matches('/'));

    Ok(stdout
        .lines()
        .filter_map(|line| line.trim().strip_prefix(&prefix).map(String::from))
        .collect())
}

pub async fn push(device: &str, local: &str, remote: &str) -> Result<()> {
    adb(device, &["push", local, remote]).await.map(|_| ())
}

pub async fn rm(device: &str, path: &str) -> Result<()> {
    adb(device, &["shell", "rm", "-f", &shell_escape(path)])
        .await
        .map(|_| ())
}

pub async fn rmdir(device: &str, path: &str) -> Result<()> {
    adb(device, &["shell", "rmdir", &shell_escape(path)])
        .await
        .map(|_| ())
}

use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum StorageType {
    Internal,
    SdCard,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StorageInfo {
    pub name: String,
    pub path: String,
    pub storage_type: StorageType,
}

pub async fn get_device_storages(device: &str) -> Result<Vec<StorageInfo>> {
    let mut storages = Vec::new();

    match adb(device, &["shell", "ls", "/storage"]).await {
        Ok(output) => {
            for line in output.lines() {
                let name = line.trim();
                if name.is_empty() || name == "self" {
                    continue;
                }
                if name == "emulated" {
                    storages.push(StorageInfo {
                        name: "Internal Storage".to_string(),
                        path: "/storage/emulated/0/Music".to_string(),
                        storage_type: StorageType::Internal,
                    });
                } else {
                    storages.push(StorageInfo {
                        name: format!("SD Card ({})", name),
                        path: format!("/storage/{}/Music", name),
                        storage_type: StorageType::SdCard,
                    });
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to list /storage via ADB: {e}");
        }
    }

    if !storages.iter().any(|s| matches!(s.storage_type, StorageType::Internal)) {
        storages.insert(0, StorageInfo {
            name: "Internal Storage (Legacy)".to_string(),
            path: "/sdcard/Music".to_string(),
            storage_type: StorageType::Internal,
        });
    }

    Ok(storages)
}



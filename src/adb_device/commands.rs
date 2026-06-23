use std::{collections::HashSet, io::ErrorKind};

use anyhow::{Context, Result};
use serde::Serialize;
use tokio::process::Command;

#[derive(Debug)]
pub enum AdbError {
    NotAvailable,
    Other(anyhow::Error),
}

impl std::fmt::Display for AdbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotAvailable => write!(f, "adb binary not found in PATH"),
            Self::Other(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for AdbError {}

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

pub async fn list_devices() -> Result<HashSet<String>, AdbError> {
    let output = Command::new("adb")
        .arg("devices")
        .output()
        .await
        .map_err(|e| {
            if e.kind() == ErrorKind::NotFound {
                AdbError::NotAvailable
            } else {
                AdbError::Other(anyhow::anyhow!("Failed to run `adb devices`: {e}"))
            }
        })?;

    if !output.status.success() {
        return Err(AdbError::Other(anyhow::anyhow!(
            "adb devices: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
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

fn shell_escape(s: &str) -> String {
    let mut escaped = String::with_capacity(s.len() + 2);
    escaped.push('\'');
    for c in s.chars() {
        if c == '\'' {
            escaped.push_str("'\\''");
        } else {
            escaped.push(c);
        }
    }
    escaped.push('\'');
    escaped
}

pub async fn ensure_dir(device: &str, dir: &str) -> Result<()> {
    adb(device, &["shell", "mkdir", "-p", &shell_escape(dir)]).await?;
    Ok(())
}

pub async fn list_files(device: &str, dir: &str) -> Result<HashSet<String>> {
    match adb(device, &["shell", "find", &shell_escape(dir), "-type", "f"]).await {
        Ok(stdout) => {
            let prefix = format!("{}/", dir.trim_end_matches('/'));
            Ok(stdout
                .lines()
                .filter_map(|line| line.trim().strip_prefix(&prefix).map(String::from))
                .collect())
        }
        Err(e) => {
            tracing::warn!(device, dir, error = %e, "Failed to list remote files via ADB");
            Ok(HashSet::new())
        }
    }
}

pub async fn push(device: &str, local: &str, remote: &str) -> Result<()> {
    adb(device, &["push", local, remote]).await?;
    Ok(())
}

pub async fn delete_file(device: &str, path: &str) -> Result<()> {
    adb(device, &["shell", "rm", "-f", &shell_escape(path)]).await?;
    Ok(())
}

pub async fn delete_dir(device: &str, path: &str) -> Result<()> {
    adb(device, &["shell", "rmdir", &shell_escape(path)]).await?;
    Ok(())
}

use utoipa::ToSchema;

#[derive(Serialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum StorageType {
    Internal,
    SdCard,
}

#[derive(Serialize, Debug, Clone, ToSchema)]
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

    if !storages
        .iter()
        .any(|s| matches!(s.storage_type, StorageType::Internal))
    {
        storages.insert(
            0,
            StorageInfo {
                name: "Internal Storage (Legacy)".to_string(),
                path: "/sdcard/Music".to_string(),
                storage_type: StorageType::Internal,
            },
        );
    }

    Ok(storages)
}

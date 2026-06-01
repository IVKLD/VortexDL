use std::collections::HashSet;

use anyhow::{Context, Result};
use tokio::process::Command;

async fn adb(device: &str, args: &[&str]) -> Result<String> {
    let output = Command::new("adb")
        .args([&["-s", device], args].concat())
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

pub async fn list_connected_devices() -> Result<HashSet<String>> {
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

pub async fn ensure_remote_dir(device: &str, dir: &str) -> Result<()> {
    adb(device, &["shell", "mkdir", "-p", &shell_escape(dir)]).await?;
    Ok(())
}

pub async fn list_remote_files(device: &str, dir: &str) -> Result<HashSet<String>> {
    let escaped = shell_escape(dir);
    let result = adb(device, &["shell", "find", &escaped, "-type", "f"]).await;

    let stdout = match result {
        Ok(s) => s,
        Err(_) => return Ok(HashSet::new()),
    };

    let prefix = dir.strip_suffix('/').unwrap_or(dir);

    Ok(stdout
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            line.strip_prefix(prefix)?
                .strip_prefix('/')
                .map(|s| s.to_string())
        })
        .collect())
}

pub async fn push(device: &str, local: &str, remote: &str) -> Result<()> {
    adb(device, &["push", local, remote]).await?;
    Ok(())
}

pub async fn rm(device: &str, path: &str) -> Result<()> {
    adb(device, &["shell", "rm", "-f", &shell_escape(path)]).await?;
    Ok(())
}

pub async fn rmdir(device: &str, path: &str) -> Result<()> {
    adb(device, &["shell", "rmdir", &shell_escape(path)]).await?;
    Ok(())
}

pub async fn media_scan(device: &str, path: &str) -> Result<()> {
    let escaped = shell_escape(path);
    let _ = adb(device, &["shell", "cmd", "media", "scan-file", &escaped]).await;

    let url = shell_escape(&format!("file://{path}"));
    let _ = adb(
        device,
        &[
            "shell",
            "am",
            "broadcast",
            "-a",
            "android.intent.action.MEDIA_SCANNER_SCAN_FILE",
            "-d",
            &url,
        ],
    )
    .await;

    Ok(())
}

pub async fn dir_scan(device: &str, dir: &str) -> Result<()> {
    let url = shell_escape(&format!("file://{dir}"));
    let _ = adb(
        device,
        &[
            "shell",
            "am",
            "broadcast",
            "-a",
            "android.intent.action.MEDIA_SCANNER_SCAN_DIR",
            "-d",
            &url,
        ],
    )
    .await;
    Ok(())
}

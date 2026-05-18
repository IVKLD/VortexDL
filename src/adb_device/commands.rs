use std::collections::HashSet;

use anyhow::{Context, Result};
use tokio::process::Command;

pub fn escape_shell_arg(arg: &str) -> String {
    let escaped = arg.replace('\'', "'\\''");
    format!("'{}'", escaped)
}

pub async fn list_connected_devices() -> Result<HashSet<String>> {
    let output = Command::new("adb")
        .arg("devices")
        .output()
        .await
        .context("Failed to run `adb devices`")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("adb devices exited with error: {stderr}");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let devices = stdout
        .lines()
        .skip(1)
        .filter_map(parse_device_line)
        .collect();

    Ok(devices)
}

fn parse_device_line(line: &str) -> Option<String> {
    let mut parts = line.split_whitespace();
    let id = parts.next()?;
    let state = parts.next()?;
    (state == "device").then(|| id.to_string())
}

pub async fn ensure_remote_dir(device_id: &str, remote_dir: &str) -> Result<()> {
    let escaped_dir = escape_shell_arg(remote_dir);
    let output = Command::new("adb")
        .args(["-s", device_id, "shell", "mkdir", "-p", &escaped_dir])
        .output()
        .await
        .context("Failed to run adb shell mkdir")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        anyhow::bail!("{stderr}");
    }

    Ok(())
}

pub async fn list_remote_files(device_id: &str, remote_dir: &str) -> Result<HashSet<String>> {
    let escaped_dir = escape_shell_arg(remote_dir);
    let output = Command::new("adb")
        .args(["-s", device_id, "shell", "find", &escaped_dir, "-type", "f"])
        .output()
        .await
        .context("Failed to run adb shell find")?;

    if !output.status.success() {
        return Ok(HashSet::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut files = HashSet::new();
    let prefix = if remote_dir.ends_with('/') {
        remote_dir.to_string()
    } else {
        format!("{remote_dir}/")
    };

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(rel) = line.strip_prefix(&prefix) {
            files.insert(rel.to_string());
        }
    }

    Ok(files)
}

pub async fn run_adb_push(device_id: &str, local_path: &str, remote_path: &str) -> Result<()> {
    let output = Command::new("adb")
        .args(["-s", device_id, "push", local_path, remote_path])
        .output()
        .await
        .context("adb push spawn failed")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        anyhow::bail!("{stderr}");
    }

    Ok(())
}

pub async fn run_adb_rm(device_id: &str, remote_path: &str) -> Result<()> {
    let escaped_path = escape_shell_arg(remote_path);
    let output = Command::new("adb")
        .args(["-s", device_id, "shell", "rm", "-f", &escaped_path])
        .output()
        .await
        .context("adb rm spawn failed")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        anyhow::bail!("{stderr}");
    }

    Ok(())
}

pub async fn run_adb_rmdir(device_id: &str, remote_dir: &str) -> Result<()> {
    let escaped_dir = escape_shell_arg(remote_dir);
    let output = Command::new("adb")
        .args(["-s", device_id, "shell", "rmdir", &escaped_dir])
        .output()
        .await
        .context("adb rmdir spawn failed")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        anyhow::bail!("{stderr}");
    }

    Ok(())
}

pub async fn trigger_media_scan(device_id: &str, remote_path: &str) -> Result<()> {
    let escaped_path = escape_shell_arg(remote_path);
    let _ = Command::new("adb")
        .args([
            "-s",
            device_id,
            "shell",
            "cmd",
            "media",
            "scan-file",
            &escaped_path,
        ])
        .output()
        .await;

    let escaped_url = escape_shell_arg(&format!("file://{remote_path}"));
    let _ = Command::new("adb")
        .args([
            "-s",
            device_id,
            "shell",
            "am",
            "broadcast",
            "-a",
            "android.intent.action.MEDIA_SCANNER_SCAN_FILE",
            "-d",
            &escaped_url,
        ])
        .output()
        .await;

    Ok(())
}

pub async fn trigger_directory_scan(device_id: &str, remote_dir: &str) -> Result<()> {
    let escaped_url = escape_shell_arg(&format!("file://{remote_dir}"));
    let _ = Command::new("adb")
        .args([
            "-s",
            device_id,
            "shell",
            "am",
            "broadcast",
            "-a",
            "android.intent.action.MEDIA_SCANNER_SCAN_DIR",
            "-d",
            &escaped_url,
        ])
        .output()
        .await;

    Ok(())
}

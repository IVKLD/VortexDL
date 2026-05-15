use std::collections::HashSet;

use anyhow::{Context, Result};
use tokio::process::Command;

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

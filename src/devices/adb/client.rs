use std::{collections::HashSet, io::ErrorKind, sync::OnceLock};

use anyhow::Result;
use tokio::process::Command;

use crate::devices::error::DeviceError;

static ADB_DIR_INIT: OnceLock<()> = OnceLock::new();

fn ensure_android_dir() {
    ADB_DIR_INIT.get_or_init(|| {
        if let Ok(android_home) = std::env::var("ANDROID_USER_HOME") {
            let _ = std::fs::create_dir_all(&android_home);
        } else if let Some(home) = dirs::home_dir() {
            let _ = std::fs::create_dir_all(home.join(".android"));
        }
    });
}

/// Executes the `adb` CLI binary with raw arguments and captures stdout/stderr.
pub async fn raw(args: &[&str]) -> Result<std::process::Output, DeviceError> {
    ensure_android_dir();

    let mut cmd = Command::new("adb");
    cmd.args(args).output().await.map_err(|e| {
        if e.kind() == ErrorKind::NotFound {
            DeviceError::NotAvailable
        } else {
            DeviceError::Other(anyhow::anyhow!(
                "Failed to run adb {}: {e}",
                args.first().unwrap_or(&"")
            ))
        }
    })
}

/// Executes an ADB command targeted at a specific device via `-s <device>`.
pub async fn exec(device: &str, args: &[&str]) -> Result<String, DeviceError> {
    let mut final_args = vec!["-s", device];
    final_args.extend_from_slice(args);
    let output = raw(&final_args).await?;

    if !output.status.success() {
        return Err(DeviceError::Other(anyhow::anyhow!(
            "{}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub fn shell_escape(s: &str) -> String {
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

/// Parses ADB device list output format (`<id>\t<state>` per line).
/// Used by both `adb devices` CLI output and `host:track-devices` protocol packets.
pub fn parse_device_list(text: &str) -> HashSet<String> {
    text.lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let id = parts.next()?;
            (parts.next()? == "device").then(|| id.to_string())
        })
        .collect()
}

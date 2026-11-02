use std::collections::HashSet;

use crate::devices::{DeviceError, adb};

pub async fn ensure_dir(device: &str, dir: &str) -> Result<(), DeviceError> {
    adb::exec_shell(device, &format!("mkdir -p {}", adb::shell_escape(dir))).await?;
    Ok(())
}

pub async fn list_files(device: &str, dir: &str) -> Result<HashSet<String>, DeviceError> {
    let stdout =
        adb::exec_shell(device, &format!("find {} -type f", adb::shell_escape(dir))).await?;

    let prefix = format!("{}/", dir.trim_end_matches('/'));
    Ok(stdout
        .lines()
        .filter_map(|line| line.trim().strip_prefix(&prefix).map(String::from))
        .collect())
}

pub async fn push(device: &str, local: &str, remote: &str) -> Result<(), DeviceError> {
    adb::exec(device, &["push", local, remote]).await?;
    Ok(())
}

pub async fn delete_file(device: &str, path: &str) -> Result<(), DeviceError> {
    adb::exec_shell(device, &format!("rm -f {}", adb::shell_escape(path))).await?;
    Ok(())
}

pub async fn delete_dir(device: &str, path: &str) -> Result<(), DeviceError> {
    adb::exec_shell(device, &format!("rmdir {}", adb::shell_escape(path))).await?;
    Ok(())
}

pub async fn sync_device_fs(device: &str) -> Result<(), DeviceError> {
    adb::exec_shell(device, "sync").await?;
    Ok(())
}

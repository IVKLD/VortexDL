use std::collections::HashSet;

use crate::adb::{
    error::AdbError,
    transport::{adb, shell_escape},
};

pub async fn ensure_dir(device: &str, dir: &str) -> Result<(), AdbError> {
    adb(
        device,
        &["shell", &format!("mkdir -p {}", shell_escape(dir))],
    )
    .await?;
    Ok(())
}

pub async fn list_files(device: &str, dir: &str) -> Result<HashSet<String>, AdbError> {
    let stdout = adb(
        device,
        &["shell", &format!("find {} -type f", shell_escape(dir))],
    )
    .await?;

    let prefix = format!("{}/", dir.trim_end_matches('/'));
    Ok(stdout
        .lines()
        .filter_map(|line| line.trim().strip_prefix(&prefix).map(String::from))
        .collect())
}

pub async fn push(device: &str, local: &str, remote: &str) -> Result<(), AdbError> {
    adb(device, &["push", local, remote]).await?;
    Ok(())
}

pub async fn delete_file(device: &str, path: &str) -> Result<(), AdbError> {
    adb(device, &["shell", &format!("rm -f {}", shell_escape(path))]).await?;
    Ok(())
}

pub async fn delete_dir(device: &str, path: &str) -> Result<(), AdbError> {
    adb(device, &["shell", &format!("rmdir {}", shell_escape(path))]).await?;
    Ok(())
}

pub async fn sync_device_fs(device: &str) -> Result<(), AdbError> {
    adb(device, &["shell", "sync"]).await?;
    Ok(())
}

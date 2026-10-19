use std::collections::HashSet;

use crate::adb::{
    error::AdbError,
    transport::{parse_device_list, run_adb_raw},
};

pub async fn list_devices() -> Result<HashSet<String>, AdbError> {
    let output = run_adb_raw(&["devices"]).await?;

    if !output.status.success() {
        return Err(AdbError::Other(anyhow::anyhow!(
            "adb devices: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let body = stdout.lines().skip(1).collect::<Vec<_>>().join("\n");
    Ok(parse_device_list(&body))
}

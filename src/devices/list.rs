use std::collections::HashSet;

use super::{DeviceError, adb};

pub async fn list_devices() -> Result<HashSet<String>, DeviceError> {
    let output = adb::raw(&["devices"]).await?;

    if !output.status.success() {
        return Err(DeviceError::Other(anyhow::anyhow!(
            "adb devices: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let body = stdout.lines().skip(1).collect::<Vec<_>>().join("\n");
    Ok(adb::parse_device_list(&body))
}

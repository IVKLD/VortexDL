use std::collections::HashSet;

use crate::devices::{DeviceError, adb};

pub async fn list_devices() -> Result<HashSet<String>, DeviceError> {
    match adb::list_devices_tcp().await {
        Ok(devices) => Ok(devices),
        Err(DeviceError::NotAvailable) => Err(DeviceError::NotAvailable),
        Err(_) => {
            let output = adb::raw(&["devices"]).await?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            let body = stdout.lines().skip(1).collect::<Vec<_>>().join("\n");
            Ok(adb::parse_device_list(&body))
        }
    }
}

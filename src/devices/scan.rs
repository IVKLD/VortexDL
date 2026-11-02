use crate::devices::{error::DeviceError, players};

pub async fn scan_device(device: &str) -> Result<(), DeviceError> {
    let scanner = players::get_scanner_for_device(device);
    scanner.scan(device).await
}

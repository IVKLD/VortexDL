use crate::devices::{adb, error::DeviceError, registry::SyncGuard};

#[derive(Clone, Copy, Debug, Default)]
pub struct GenericMediaScanner;

impl GenericMediaScanner {
    pub async fn scan(&self, device: &str) -> Result<(), DeviceError> {
        let _guard = SyncGuard::try_acquire(device).ok_or(DeviceError::AlreadyInProgress)?;
        adb::exec_shell(
            device,
            "am broadcast -a android.intent.action.MEDIA_SCANNER_SCAN_FILE -d file:///sdcard/Music",
        )
        .await?;
        Ok(())
    }
}

pub mod cayin;
pub mod generic;

pub use cayin::CayinN3UltraScanner;
pub use generic::GenericMediaScanner;

use crate::devices::error::DeviceError;

#[derive(Clone, Copy, Debug)]
pub enum PlayerScanner {
    Cayin(CayinN3UltraScanner),
    Generic(GenericMediaScanner),
}

impl PlayerScanner {
    pub async fn scan(&self, device_id: &str) -> Result<(), DeviceError> {
        match self {
            Self::Cayin(s) => s.scan(device_id).await,
            Self::Generic(s) => s.scan(device_id).await,
        }
    }
}

pub fn get_scanner_for_device(_device_id: &str) -> PlayerScanner {
    PlayerScanner::Cayin(CayinN3UltraScanner)
}

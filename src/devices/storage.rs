use anyhow::Result;
use serde::Serialize;
use utoipa::ToSchema;

use super::adb;

#[derive(Serialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum StorageType {
    Internal,
    SdCard,
}

#[derive(Serialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StorageInfo {
    pub name: String,
    pub path: String,
    pub storage_type: StorageType,
}

pub async fn get_device_storages(device: &str) -> Result<Vec<StorageInfo>> {
    let mut storages = Vec::new();

    match adb::exec(device, &["shell", "ls /storage"]).await {
        Ok(output) => {
            for line in output.lines() {
                let name = line.trim();
                if name.is_empty() || name == "self" {
                    continue;
                }
                if name == "emulated" {
                    storages.push(StorageInfo {
                        name: "Internal Storage".to_string(),
                        path: "/storage/emulated/0/Music".to_string(),
                        storage_type: StorageType::Internal,
                    });
                } else {
                    storages.push(StorageInfo {
                        name: format!("SD Card ({name})"),
                        path: format!("/storage/{name}/Music"),
                        storage_type: StorageType::SdCard,
                    });
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to list /storage via ADB: {e}");
        }
    }

    if !storages
        .iter()
        .any(|s| matches!(s.storage_type, StorageType::Internal))
    {
        storages.insert(
            0,
            StorageInfo {
                name: "Internal Storage (Legacy)".to_string(),
                path: "/sdcard/Music".to_string(),
                storage_type: StorageType::Internal,
            },
        );
    }

    Ok(storages)
}

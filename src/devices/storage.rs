use anyhow::Result;
use serde::Serialize;
use utoipa::ToSchema;

use crate::devices::adb;

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

    let probe_script = r#"
for d in /storage/*; do
  [ ! -d "$d" ] && continue
  name=$(basename "$d")
  [ "$name" = "self" ] && continue
  if [ "$name" = "emulated" ]; then
    root="/storage/emulated/0"
  else
    root="$d"
  fi
  echo "ROOT:$name:$root"
  for sub in Music music Audio; do
    [ -d "$root/$sub" ] && echo "SUB:$name:$root/$sub"
  done
done
"#;

    match adb::exec_shell(device, probe_script.trim()).await {
        Ok(output) => {
            let mut current_root: Option<(String, String)> = None;
            let mut found_subs = Vec::new();

            let process_volume =
                |root_entry: Option<(String, String)>,
                 subs: Vec<String>,
                 storages: &mut Vec<StorageInfo>| {
                    if let Some((name, root)) = root_entry {
                        let is_internal = name == "emulated";
                        let storage_type = if is_internal {
                            StorageType::Internal
                        } else {
                            StorageType::SdCard
                        };

                        if subs.is_empty() {
                            let default_music = format!("{root}/Music");
                            let default_label = if is_internal {
                                "Internal Storage (Music)".to_string()
                            } else {
                                format!("SD Card ({name}/Music)")
                            };
                            storages.push(StorageInfo {
                                name: default_label,
                                path: default_music,
                                storage_type: storage_type.clone(),
                            });
                        } else {
                            for sub_path in subs {
                                let folder_name = sub_path.rsplit('/').next().unwrap_or("Music");
                                let label = if is_internal {
                                    format!("Internal Storage ({folder_name})")
                                } else {
                                    format!("SD Card ({name}/{folder_name})")
                                };
                                storages.push(StorageInfo {
                                    name: label,
                                    path: sub_path,
                                    storage_type: storage_type.clone(),
                                });
                            }
                        }

                        let root_label = if is_internal {
                            "Internal Storage (Root)".to_string()
                        } else {
                            format!("SD Card ({name})")
                        };
                        storages.push(StorageInfo {
                            name: root_label,
                            path: root,
                            storage_type,
                        });
                    }
                };

            for line in output.lines() {
                let line = line.trim();
                if let Some(rest) = line.strip_prefix("ROOT:") {
                    process_volume(
                        current_root.take(),
                        std::mem::take(&mut found_subs),
                        &mut storages,
                    );
                    if let Some((name, root)) = rest.split_once(':') {
                        current_root = Some((name.to_string(), root.to_string()));
                    }
                } else if let Some(rest) = line.strip_prefix("SUB:")
                    && let Some((_name, sub)) = rest.split_once(':')
                {
                    found_subs.push(sub.to_string());
                }
            }
            process_volume(current_root, found_subs, &mut storages);
        }
        Err(e) => {
            tracing::warn!("Failed to query /storage via ADB: {e}");
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

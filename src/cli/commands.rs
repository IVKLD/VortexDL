use std::collections::HashSet;

use anyhow::{Result, anyhow, bail};
use url::Url;

use crate::{
    app::AppState,
    devices::{self, DeviceError},
    downloader,
};

async fn list_connected_devices() -> Result<Option<HashSet<String>>> {
    match devices::list_devices().await {
        Ok(devs) if devs.is_empty() => {
            println!("No connected ADB devices found.");
            Ok(None)
        }
        Ok(devs) => Ok(Some(devs)),
        Err(DeviceError::NotAvailable) => {
            bail!("adb binary not found in PATH. Please install adb and ensure it is accessible.");
        }
        Err(e) => Err(anyhow!(e)),
    }
}

pub async fn run_scan_player() -> Result<()> {
    let Some(connected) = list_connected_devices().await? else {
        return Ok(());
    };

    for id in &connected {
        println!("Scanning music library on device: {id}...");
        match devices::scan_device(id).await {
            Ok(()) => println!("Scan triggered successfully on device: {id}"),
            Err(e) => eprintln!("Failed to scan device {id}: {e}"),
        }
    }

    Ok(())
}

pub async fn run_sync_player(state: &AppState) -> Result<()> {
    let settings = state.settings.read().await;
    if !settings.adb.enabled {
        bail!("ADB is disabled in settings. Please enable it in settings to sync.");
    }

    let Some(connected) = list_connected_devices().await? else {
        return Ok(());
    };

    let mut synced_any = false;
    for id in &connected {
        if let Some(cfg) = settings.adb.find_enabled_device(id) {
            println!("Syncing with device: {} -> {}", id, cfg.remote_music_dir);
            devices::sync_device(id, &cfg.remote_music_dir, state.library.clone(), true).await?;
            synced_any = true;
        } else {
            println!(
                "Device {} is connected but either disabled or not configured in settings.",
                id
            );
        }
    }

    if !synced_any {
        println!("No configured and enabled ADB devices are currently connected.");
    }

    Ok(())
}

pub async fn run_download(state: &AppState, url: &Url, proxies: &[String]) -> Result<()> {
    let ctx = state.downloader_context();

    if !proxies.is_empty() {
        let mut current_settings = ctx.settings.read().await.clone();
        current_settings.network.use_proxy = true;
        current_settings.network.fallback_proxies = proxies.to_vec();
        ctx.settings.update(current_settings).await?;
    }

    downloader::run_download_pipeline(&ctx, url).await
}

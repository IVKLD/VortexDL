use std::{collections::HashSet, net::IpAddr, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use url::Url;

use crate::{
    adb,
    api::state::AppState,
    downloader::{self, Context},
    settings::UserSettings,
};

#[derive(Parser, Debug)]
#[command(name = "vortex-dl")]
#[command(version)]
#[command(about = "VortexDL: High-performance SoundCloud downloader with intelligent sync", long_about = None)]
#[command(disable_version_flag = true)]
#[command(arg_required_else_help = true)]
pub struct Args {
    #[arg(
        short = 'v',
        long = "version",
        action = clap::ArgAction::Version,
        help = "Print version information"
    )]
    pub version: (),
    #[arg(help = "The SoundCloud URL to download (track, playlist, or user likes)")]
    pub url: Option<Url>,

    #[arg(
        short,
        long,
        env = "VORTEX_OUTPUT_DIR",
        help = "Directory where the music will be saved"
    )]
    pub output: Option<PathBuf>,

    #[arg(
        long,
        default_value_t = false,
        help = "Run the application as a REST API server"
    )]
    pub serve: bool,

    #[arg(
        long,
        default_value_t = false,
        help = "Sync music files with configured and connected ADB devices / music player"
    )]
    pub sync_player: bool,

    #[arg(long, default_value_t = false, help = "Scan music library on player")]
    pub scan_player: bool,

    #[arg(
        long,
        default_value = "127.0.0.1",
        help = "Host address to bind the server to"
    )]
    pub host: IpAddr,

    #[arg(
        long,
        env = "VORTEX_PORT",
        default_value_t = 3200,
        help = "Port to listen on for the REST API and WebUI"
    )]
    pub port: u16,

    #[arg(
        long,
        value_delimiter = ',',
        help = "Fallback proxy URLs (comma-separated)"
    )]
    pub proxies: Vec<String>,
}

impl Args {
    pub fn resolve_output_dir(&self, settings: &UserSettings) -> PathBuf {
        self.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(&settings.downloads.output_path))
    }
}

async fn list_connected_devices() -> Result<Option<HashSet<String>>> {
    match adb::list_devices().await {
        Ok(devices) if devices.is_empty() => {
            println!("No connected ADB devices found.");
            Ok(None)
        }
        Ok(devices) => Ok(Some(devices)),
        Err(adb::AdbError::NotAvailable) => {
            println!(
                "adb binary not found in PATH. Please install adb and ensure it is accessible."
            );
            Ok(None)
        }
        Err(e) => Err(anyhow::anyhow!(e)),
    }
}

pub async fn run_scan_player() -> Result<()> {
    let Some(connected) = list_connected_devices().await? else {
        return Ok(());
    };

    for id in &connected {
        println!("Scanning music library on device: {id}...");
        match adb::scan_device(id).await {
            Ok(()) => println!("Scan triggered successfully on device: {id}"),
            Err(e) => eprintln!("Failed to scan device {id}: {e}"),
        }
    }

    Ok(())
}

pub async fn run_sync_player(state: &AppState) -> Result<()> {
    let settings = state.settings.read().await;
    if !settings.adb.enabled {
        println!("ADB is disabled in settings. Please enable it to sync.");
        return Ok(());
    }

    let Some(connected) = list_connected_devices().await? else {
        return Ok(());
    };

    let mut synced_any = false;
    for id in &connected {
        if let Some(cfg) = settings.adb.find_enabled_device(id) {
            println!("Syncing with device: {} -> {}", id, cfg.remote_music_dir);
            adb::sync_device(id, &cfg.remote_music_dir, state.storage.clone(), true).await?;
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
    let ctx = Context::from_state(state);

    if !proxies.is_empty() {
        let mut current_settings = ctx.settings.read().await.clone();
        current_settings.network.fallback_proxies = proxies.to_vec();
        ctx.settings.update(current_settings).await?;
    }

    downloader::run_download_pipeline(&ctx, url).await
}

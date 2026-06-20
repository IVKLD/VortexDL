use anyhow::Result;
use clap::Parser;

use crate::{
    adb_device,
    api::{self, state::AppState},
    downloader::{self, Context},
    settings::UserSettings,
    types::SyncMode,
};

#[derive(Parser, Debug)]
#[command(name = "vortex-dl")]
#[command(about = "VortexDL: High-performance SoundCloud downloader with intelligent sync", long_about = None)]
pub struct Args {
    #[arg(help = "The SoundCloud URL to download (track, playlist, or user likes)")]
    pub url: Option<String>,

    #[arg(short, long, help = "Directory where the music will be saved")]
    pub output: Option<String>,

    #[arg(long, default_value_t = SyncMode::Silent, help = "Sync behavior: silent (accumulate), archive (move removed to Archive), or full (delete removed)")]
    pub sync_mode: SyncMode,

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

    #[arg(
        long,
        default_value = "127.0.0.1",
        help = "Host address to bind the server to"
    )]
    pub host: String,

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
    pub fn resolve_output_dir(&self, settings: &UserSettings) -> String {
        self.output
            .as_ref()
            .unwrap_or(&settings.downloads.output_path)
            .clone()
    }
}

pub async fn execute_app(state: AppState, args: Args) -> Result<()> {
    if args.serve {
        api::run_server(state, &args).await
    } else if args.sync_player {
        run_cli_sync(state).await
    } else if let Some(ref url) = args.url {
        run_cli_download(state, url, &args).await
    } else {
        Ok(())
    }
}

async fn run_cli_sync(state: AppState) -> Result<()> {
    let settings = state.settings.read().await;
    if !settings.adb.enabled {
        println!("ADB is disabled in settings. Please enable it to sync.");
        return Ok(());
    }

    let connected = match adb_device::list_devices().await {
        Ok(devices) => devices,
        Err(adb_device::AdbError::NotAvailable) => {
            println!("adb binary not found in PATH. Please install adb and ensure it is accessible.");
            return Ok(());
        }
        Err(e) => return Err(anyhow::anyhow!(e)),
    };
    if connected.is_empty() {
        println!("No connected ADB devices found.");
        return Ok(());
    }

    let mut synced_any = false;
    for id in &connected {
        if let Some(cfg) = settings
            .adb
            .devices
            .iter()
            .find(|d| d.enabled && d.device_id == *id)
        {
            println!("Syncing with device: {} -> {}", id, cfg.remote_music_dir);
            adb_device::sync_device(id, &cfg.remote_music_dir, state.storage.clone()).await?;
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

async fn run_cli_download(state: AppState, url: &str, args: &Args) -> Result<()> {
    let ctx = Context::from_state(&state);

    let mut current_settings = ctx.settings.read().await.clone();
    current_settings.downloads.sync_mode = args.sync_mode;
    if !args.proxies.is_empty() {
        current_settings.network.fallback_proxies = args.proxies.clone();
    }
    ctx.settings.update_in_memory(current_settings).await;

    downloader::download(&ctx, url).await?;
    Ok(())
}

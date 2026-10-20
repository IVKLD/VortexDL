use std::{net::IpAddr, path::PathBuf};

use clap::Parser;
use url::Url;

use crate::settings::UserSettings;

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

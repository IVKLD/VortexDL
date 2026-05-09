use clap::Parser;

use crate::models::SyncMode;

#[derive(Parser, Debug)]
#[command(name = "vortex-dl")]
#[command(about = "VortexDL: High-performance SoundCloud downloader with intelligent sync", long_about = None)]
pub struct Args {
    #[arg(help = "The SoundCloud URL to download (track, playlist, or user likes)")]
    pub url: Option<String>,

    #[arg(
        short,
        long,
        default_value = "./downloads",
        help = "Directory where the music will be saved"
    )]
    pub output: String,

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
        default_value = "0.0.0.0",
        help = "Host address to bind the server to"
    )]
    pub host: String,

    #[arg(
        long,
        default_value_t = 3200,
        help = "Port to listen on for the REST API and WebUI"
    )]
    pub port: u16,
}

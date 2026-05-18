use anyhow::Result;
use clap::{CommandFactory, Parser};

use crate::{
    api::{self, state::AppState},
    downloader::{self, Context},
    models::SyncMode,
    settings::UserSettings,
};

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
}

impl Args {
    pub fn resolve_output_dir(&self, settings: &UserSettings) -> String {
        if !self.output.is_empty() {
            self.output.clone()
        } else {
            settings.downloads.output_path.clone()
        }
    }
}

pub async fn execute_app(state: AppState, args: Args) -> Result<()> {
    if args.serve {
        api::run_server(state, &args).await
    } else if let Some(ref url) = args.url {
        run_cli_download(state, url, &args).await
    } else {
        Args::command().print_help()?;
        println!();
        Ok(())
    }
}

async fn run_cli_download(state: AppState, url: &str, args: &Args) -> Result<()> {
    let ctx = Context {
        storage: state.storage.clone(),
        client: state.client.clone(),
        http: state.http.clone(),
        dm: None,
        settings: state.settings.clone(),
    };

    let mut current_settings = ctx.settings.read().await.clone();
    current_settings.downloads.sync_mode = args.sync_mode.clone();
    ctx.settings.update(current_settings).await?;

    downloader::download(&ctx, url).await?;
    Ok(())
}

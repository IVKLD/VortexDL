use std::{fs, sync::Arc};

use anyhow::Result;
use clap::Parser;
use mimalloc::MiMalloc;
use tokio::sync::RwLock;

use crate::{
    api::state::AppState, cli::Args, settings::SettingsManager, storage::MusicStorage,
    ui::create_standalone_spinner, utils::soundcloud,
};

mod adb;
mod api;
mod backup;
mod cli;
mod constants;
mod database;
mod download_manager;
mod downloader;
mod settings;
mod storage;
mod types;
mod ui;
mod utils;
mod watchdog;

#[global_allocator]
static ALLOC: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() -> Result<()> {
    utils::setup_tracing();
    let args = Args::parse();

    let state = init_app(&args).await?;
    handle_args(state, args).await
}

async fn handle_args(state: AppState, args: Args) -> Result<()> {
    if args.serve {
        api::run_server(state, &args).await
    } else if args.scan_player {
        cli::run_scan_player().await
    } else if args.sync_player {
        cli::run_sync_player(&state).await
    } else if let Some(ref url) = args.url {
        cli::run_download(&state, url, &args.proxies).await
    } else {
        Ok(())
    }
}

async fn init_app(args: &Args) -> Result<AppState> {
    database::init()?;
    let mut settings = database::get_settings()?;
    let output_dir = args.resolve_output_dir(&settings);
    settings.downloads.output_path = output_dir.to_string_lossy().to_string();
    fs::create_dir_all(&output_dir)?;

    let pb = create_standalone_spinner("Initializing SoundCloud...");
    let storage = Arc::new(RwLock::new(MusicStorage::default()));
    let settings_manager = SettingsManager::new(settings.clone());
    let client = soundcloud::ClientBuilder::new(&settings)
        .with_settings_manager(settings_manager.clone())
        .build()
        .await?;
    pb.finish_with_message("SoundCloud ready");

    let state = AppState::new(client, storage.clone(), settings_manager);

    let pb_idx = create_standalone_spinner("Indexing local library...");
    let tracks = MusicStorage::scan_library(&output_dir).await;
    storage.write().await.update_tracks(tracks);
    pb_idx.finish_with_message("Local library indexed");

    watchdog::init(
        storage.clone(),
        state.settings.clone(),
        state.download_manager.clone(),
    )
    .await?;

    adb::init(storage, state.settings.clone());

    Ok(state)
}

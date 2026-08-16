use std::{fs, sync::Arc};

use anyhow::Result;
use clap::Parser;
use tokio::sync::RwLock;

use crate::{
    api::state::AppState, cli::Args, settings::SettingsManager, storage::MusicStorage,
    ui::create_standalone_spinner, utils::soundcloud,
};

mod adb_device;
mod api;
mod cli;
mod constants;
mod database;
mod downloader;
mod settings;
mod storage;
mod types;
mod ui;
mod utils;
mod watchdog;

#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    utils::tracing::setup();
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

    let state = AppState::from_parts(client, storage.clone(), settings_manager);

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

    adb_device::init(storage, state.settings.clone());

    cli::execute_app(state, args).await
}

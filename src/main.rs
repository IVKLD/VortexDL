use std::{fs, sync::Arc};

use anyhow::Result;
use clap::Parser;
use tokio::sync::RwLock;

use crate::{
    api::state::AppState, cli::Args, storage::MusicStorage, ui::create_standalone_spinner,
    utils::soundcloud::SoundCloudClientBuilder,
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
    if args.url.is_none() && !args.serve && !args.sync_player {
        use clap::CommandFactory;
        Args::command().print_help()?;
        println!();
        return Ok(());
    }

    utils::tracing::setup();
    database::init()?;

    let settings = database::get_settings()?;
    let output_dir = args.resolve_output_dir(&settings);
    fs::create_dir_all(&output_dir)?;

    let pb = create_standalone_spinner("Initializing SoundCloud...");
    let storage = Arc::new(RwLock::new(MusicStorage::default()));
    let settings_manager = crate::settings::SettingsManager::new(settings.clone());
    let client = SoundCloudClientBuilder::new(&settings)
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

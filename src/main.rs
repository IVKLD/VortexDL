use std::{fs, sync::Arc};

use anyhow::Result;
use clap::Parser;
use tokio::sync::RwLock;

use crate::{
    api::state::AppState, cli::Args, storage::MusicStorage, ui::create_standalone_spinner,
    utils::soundcloud::init_client,
};

mod adb_device;
mod api;
mod cli;
mod constants;
mod database;
mod downloader;
mod models;
mod settings;
mod storage;
mod ui;
mod utils;

#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() -> Result<()> {
    utils::tracing::setup();
    database::init()?;

    let args = Args::parse();

    let mut settings = database::settings::get_settings_db()?;
    let output_dir = args.resolve_output_dir(&settings);
    fs::create_dir_all(&output_dir)?;

    let storage = Arc::new(RwLock::new(MusicStorage::new(output_dir.clone())));

    let pb = create_standalone_spinner("Initializing SoundCloud...");
    let client = init_client(&mut settings).await?;
    pb.finish_with_message("SoundCloud ready");

    let state = AppState::new(client, storage.clone(), settings);

    adb_device::init(state.storage.clone(), state.settings.clone());

    tokio::spawn(async move {
        MusicStorage::run_background_indexing(storage).await;
    });

    cli::execute_app(state, args).await
}

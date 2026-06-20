use std::{fs, sync::Arc};

use anyhow::Result;
use clap::Parser;
use tokio::sync::RwLock;

use crate::{
    api::state::AppState, cli::Args, storage::MusicStorage, ui::create_standalone_spinner,
    utils::soundcloud::init_client_with_settings,
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

#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() -> Result<()> {
    utils::tracing::setup();
    database::init()?;

    let args = Args::parse();
    if args.url.is_none() && !args.serve && !args.sync_player {
        use clap::CommandFactory;
        Args::command().print_help()?;
        println!();
        return Ok(());
    }

    let settings = database::get_settings()?;
    let output_dir = args.resolve_output_dir(&settings);
    fs::create_dir_all(&output_dir)?;

    let storage = Arc::new(RwLock::new(MusicStorage::new(output_dir)));

    let pb = create_standalone_spinner("Initializing SoundCloud...");
    let client = init_client_with_settings(&settings, None).await?;
    pb.finish_with_message("SoundCloud ready");

    let state = AppState::new(Arc::new(client), storage, settings);

    let pb_idx = create_standalone_spinner("Indexing local library...");
    MusicStorage::index_library(state.storage.clone()).await;
    pb_idx.finish_with_message("Local library indexed");

    adb_device::init(state.storage.clone(), state.settings.clone());

    cli::execute_app(state, args).await
}

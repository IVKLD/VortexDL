use std::{fs, net::SocketAddr};

use anyhow::Result;
use axum::serve;
use clap::Parser;
use tokio::net::TcpListener;

use crate::{
    api::state::AppState, cli::Args, database::settings::UserSettings,
    ui::create_standalone_spinner, utils::soundcloud::init_client,
};

mod api;
mod cli;
mod constants;
mod database;
mod downloader;
mod models;
mod storage;
mod ui;
mod utils;

#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() -> Result<()> {
    setup_tracing();
    database::init()?;

    let args = Args::parse();
    let state = bootstrap_state(&args).await?;

    run_background_indexing(&state);

    execute_app(state, args).await
}

async fn bootstrap_state(args: &Args) -> Result<AppState> {
    let mut settings = database::settings::get_settings()?;
    let output_dir = resolve_output_dir(args, &settings);
    fs::create_dir_all(&output_dir)?;

    let pb = create_standalone_spinner("Initializing SoundCloud...");
    let client = init_client(&mut settings).await?;
    pb.finish_with_message("SoundCloud ready");

    Ok(AppState::new(client, output_dir, settings))
}

fn run_background_indexing(state: &AppState) {
    let storage = state.storage.clone();
    tokio::spawn(async move {
        storage::MusicStorage::run_background_indexing(storage).await;
    });
}

fn resolve_output_dir(args: &Args, settings: &UserSettings) -> String {
    if !args.output.is_empty() {
        args.output.clone()
    } else {
        settings.downloads.output_path.clone()
    }
}

async fn execute_app(state: AppState, args: Args) -> Result<()> {
    if args.serve {
        run_server(state, &args).await
    } else if let Some(ref url) = args.url {
        run_cli_download(state, url, &args).await
    } else {
        Ok(())
    }
}

fn setup_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();
}

async fn run_server(state: AppState, args: &Args) -> Result<()> {
    let router = api::build_router(state, args.serve).await;
    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;
    let listener = TcpListener::bind(addr).await?;

    println!("VortexLD running on http://{}", addr);
    serve(listener, router).await?;
    Ok(())
}

async fn run_cli_download(state: AppState, url: &str, args: &Args) -> Result<()> {
    let ctx = downloader::Context {
        storage: state.storage.clone(),
        client: state.client.clone(),
        dm: None,
        settings: state.settings.clone(),
    };

    downloader::dispatch_download(url, args.sync_mode.clone(), &ctx).await?;
    Ok(())
}

use std::{fs, net::SocketAddr, path::Path, sync::Arc};

use anyhow::Result;
use axum::serve;
use clap::Parser;
use soundcloud_rs::ClientBuilder;
use tokio::net::TcpListener;

use crate::{api::state::AppState, cli::Args, config::AppConfig, ui::create_standalone_spinner};

mod api;
mod cli;
mod config;
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

    let args = Args::parse();
    let state = bootstrap_state(&args).await?;

    execute_app(state, args).await
}

async fn bootstrap_state(args: &Args) -> Result<AppState> {
    let config = Arc::new(AppConfig::default());
    fs::create_dir_all(&args.output)?;

    let pb_client = create_standalone_spinner("Initializing SC client...");
    let client = setup_soundcloud_client(&config).await?;
    pb_client.finish_with_message("SC client initialized successfully");

    database::init()?;

    let state = AppState::new(client, config, args.output.clone());

    let pb_idx = create_standalone_spinner("Indexing local tracks...");
    state
        .storage
        .write()
        .await
        .indexing(Path::new(&args.output));
    pb_idx.finish_and_clear();

    Ok(state)
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

async fn setup_soundcloud_client(config: &AppConfig) -> Result<Arc<soundcloud_rs::Client>> {
    let client = ClientBuilder::new()
        .with_max_retries(config.max_retries)
        .with_retry_on_401(true)
        .build()
        .await?;
    Ok(Arc::new(client))
}

async fn run_server(state: AppState, args: &Args) -> Result<()> {
    let router = api::build_router(state, args.serve).await;

    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;
    let listener = TcpListener::bind(addr).await?;

    println!(
        "REST API listening on http://{}:{}/api",
        args.host, args.port
    );
    println!("WebUI available at http://{}:{}", args.host, args.port);

    serve(listener, router).await?;
    Ok(())
}

async fn run_cli_download(state: AppState, url: &str, args: &Args) -> Result<()> {
    let ctx = downloader::Context {
        storage: state.storage.clone(),
        client: state.client.clone(),
        config: state.config.clone(),
        dm: None,
    };

    downloader::dispatch_download(url, args.sync_mode.clone(), &ctx).await?;

    Ok(())
}

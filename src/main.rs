mod api;
mod cli;
mod config;
mod downloader;
mod models;
mod storage;
mod utils;

use clap::Parser;
use cli::Args;
use config::AppConfig;
use soundcloud_rs::ClientBuilder;
use std::{net::SocketAddr, path::Path, sync::Arc};
use crate::api::state::AppState;

#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let args = Args::parse();
    let config = Arc::new(AppConfig::default());

    let output_dir = args
        .output
        .unwrap_or_else(|| config.default_output_dir.clone());

    let client = Arc::new(ClientBuilder::new()
        .with_max_retries(config.max_retries)
        .with_retry_on_401(true)
        .build()
        .await?);

    let state = AppState::new(client.clone(), config.clone(), output_dir.clone());
    {
        let mut storage = state.storage.write().await;
        storage.indexing(Path::new(&output_dir));
    }

    if args.serve {
        let router = api::build_router(state.clone(), args.serve).await;

        std::fs::create_dir_all(&output_dir)?;

        let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;
        let listener = tokio::net::TcpListener::bind(addr).await?;

        println!("REST API listening on http://{}:{}", args.host, args.port);

        axum::serve(listener, router).await?;

        return Ok(());
    }

    if let Some(url) = args.url {
        let remote_ids = downloader::dispatch_download(
            &url,
            state.storage.clone(),
            &client,
            &output_dir,
            config.clone(),
            None
        ).await?;

        if args.sync {
            let storage = state.storage.write().await;
            storage.sync_storage(&remote_ids, &output_dir, &args.sync_mode).await?;
        }
    }

    Ok(())
}

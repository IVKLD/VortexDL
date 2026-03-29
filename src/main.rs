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
use std::{error::Error, net::SocketAddr, path::Path};

use crate::{
    api::state::AppState,
    models::{ResolveQuery, ResolveResponse},
    storage::MusicStorage,
};

#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let args = Args::parse();
    let config = AppConfig::default();

    let output_dir = args
        .output
        .unwrap_or_else(|| config.default_output_dir.clone());

    tracing::info!("Initializing SoundCloud client...");

    let client = ClientBuilder::new()
        .with_max_retries(config.max_retries)
        .with_retry_on_401(true)
        .build()
        .await?;

    // ── Server mode ──────────────────────────────────────────────────────────
    if args.serve {
        let state = AppState::new(client, config, output_dir.clone());
        let router = api::build_router(state);

        // Ensure the output directory exists before serving.
        std::fs::create_dir_all(&output_dir)?;

        let addr: SocketAddr = format!("0.0.0.0:{}", args.port).parse()?;
        let listener = tokio::net::TcpListener::bind(addr).await?;
        tracing::info!("REST API listening on http://{addr}");
        axum::serve(listener, router).await?;

        return Ok(());
    }

    // ── CLI mode ─────────────────────────────────────────────────────────────
    let mut music_storage = MusicStorage::new();
    music_storage.indexing(Path::new(&output_dir));

    if let Some(playlist_url) = args.playlist_url {
        downloader::download_playlist(&mut music_storage, &client, &playlist_url, &output_dir)
            .await?;
    } else if let Some(user_url) = args.user_url {
        downloader::download_likes(&mut music_storage, &client, &user_url, &output_dir, &config)
            .await?;
    } else if !output_dir.is_empty() {
        sync(&client, &output_dir).await?;
    }

    Ok(())
}

async fn sync(client: &soundcloud_rs::Client, path: &str) -> Result<(), Box<dyn Error>> {
    let query = ResolveQuery {
        url: Some(String::from("https://soundcloud.com/user-316096540/likes")),
    };

    let res: ResolveResponse = client.get("resolve", Some(&query)).await?;
    println!("{}", res.id);

    Ok(())
}

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

    if args.serve {
        let state = AppState::new(client, config, output_dir.clone());
        let router = api::build_router(state);

        std::fs::create_dir_all(&output_dir)?;

        let addr: SocketAddr = format!("0.0.0.0:{}", args.port).parse()?;
        let listener = tokio::net::TcpListener::bind(addr).await?;
        tracing::info!("REST API listening on http://{addr}");
        axum::serve(listener, router).await?;

        return Ok(());
    }

    let mut music_storage = MusicStorage::new();
    music_storage.indexing(Path::new(&output_dir));

    if let Some(url) = args.url {
        let resolve_res: ResolveResponse = client
            .get(
                "resolve",
                Some(&ResolveQuery {
                    url: Some(url.clone()),
                }),
            )
            .await?;

        let mut remote_ids = None;

        match resolve_res.kind.as_str() {
            "user" => {
                if url.ends_with("/likes") {
                    remote_ids = Some(
                        downloader::download_likes(
                            &music_storage,
                            &client,
                            &url,
                            &output_dir,
                            &config,
                        )
                        .await?,
                    );
                }
            }
            "playlist" => {
                remote_ids = Some(
                    downloader::download_playlist(&mut music_storage, &client, &url, &output_dir)
                        .await?,
                );
            }
            "track" => {
                downloader::download_track(&music_storage, &client, resolve_res.id, &output_dir)
                    .await?;
            }
            _ => {
                tracing::error!("Unsupported resource kind: {}", resolve_res.kind);
            }
        }

        if args.sync {
            if let Some(ids) = remote_ids {
                music_storage.sync_storage(&ids, &output_dir, &args.sync_mode).await?;
            }
        }
    }

    Ok(())
}

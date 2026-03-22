mod cli;
mod config;
mod downloader;
mod models;
mod storage;
mod utils;

use clap::Parser;
use cli::Args;
use config::AppConfig;
use soundcloud_rs::{Client, ClientBuilder};
use std::{error::Error, path::Path};

use crate::{
    models::{ResolveQuery, ResolveResponse},
    storage::MusicStorage,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let config = AppConfig::default();

    let output_dir = args
        .output
        .unwrap_or_else(|| config.default_output_dir.clone());

    let mut music_storage = MusicStorage::new();
    music_storage.indexing(Path::new(&output_dir));

    println!("\x1B[1;36m[SYSTEM]\x1B[0m Initializing SoundCloud client...");

    let client = ClientBuilder::new()
        .with_max_retries(config.max_retries)
        .with_retry_on_401(true)
        .build()
        .await?;

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

async fn sync(client: &Client, path: &str) -> Result<(), Box<dyn Error>> {
    let query = ResolveQuery {
        url: Some(String::from("https://soundcloud.com/user-316096540/likes")),
    };

    let res: ResolveResponse = client.get("resolve", Some(&query)).await?;
    println!("{}", res.id);

    Ok(())
}

use anyhow::Result;
use clap::Parser;
use mimalloc::MiMalloc;

use crate::cli::Args;

pub mod api;
pub mod app;
pub mod backup;
pub mod cli;
pub mod devices;
pub mod domain;
pub mod downloader;
pub mod infra;
pub mod library;
pub mod providers;
pub mod settings;
pub mod terminal;

#[global_allocator]
static ALLOC: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() -> Result<()> {
    infra::init_logging();
    let args = Args::parse();
    app::run(args).await
}

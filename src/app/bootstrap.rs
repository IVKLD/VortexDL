use std::{fs, sync::Arc};

use anyhow::Result;
use tokio::sync::RwLock;

use super::state::AppState;
use crate::{
    api,
    cli::{Args, run_download, run_scan_player, run_sync_player},
    devices, infra,
    library::{self, Library},
    providers::soundcloud::ClientBuilder,
    settings::SettingsManager,
    terminal::create_standalone_spinner,
};

pub async fn run(args: Args) -> Result<()> {
    let state = init_app(&args).await?;

    if args.serve {
        api::run_server(state, &args).await
    } else if args.scan_player {
        run_scan_player().await
    } else if args.sync_player {
        run_sync_player(&state).await
    } else if let Some(ref url) = args.url {
        run_download(&state, url, &args.proxies).await
    } else {
        Ok(())
    }
}

pub async fn init_app(args: &Args) -> Result<AppState> {
    infra::db::init()?;
    let mut settings = infra::db::get_settings()?;
    let output_dir = args.resolve_output_dir(&settings);
    settings.downloads.output_path = output_dir.to_string_lossy().to_string();
    fs::create_dir_all(&output_dir)?;

    let pb = create_standalone_spinner("Initializing SoundCloud...");
    let library = Arc::new(RwLock::new(Library::default()));
    let settings_manager = SettingsManager::new(settings.clone());
    let client = ClientBuilder::new(&settings)
        .with_settings_manager(settings_manager.clone())
        .build()
        .await?;
    pb.finish_with_message("SoundCloud ready");

    let state = AppState::new(client, library.clone(), settings_manager);

    let pb_idx = create_standalone_spinner("Indexing local library...");
    let tracks = library::scan_directory(&output_dir).await;
    library.write().await.update_tracks(tracks);
    pb_idx.finish_with_message("Local library indexed");

    super::background::start_library_watcher(
        library.clone(),
        state.settings.clone(),
        state.download_manager.clone(),
    )
    .await?;

    devices::init_device_tracker(library, state.settings.clone());

    Ok(state)
}

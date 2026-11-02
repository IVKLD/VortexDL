use std::{fs, sync::Arc};

use anyhow::Result;
use tokio::sync::RwLock;

use crate::{
    api,
    app::{background, state::AppState},
    cli::{self, Args},
    infra,
    library::{Library, LibraryService},
    providers::soundcloud::ClientBuilder,
    settings::SettingsManager,
    terminal,
};

pub async fn run(args: Args) -> Result<()> {
    let state = init_app(&args).await?;

    if args.scan_player {
        state.device_service().scan_all_players().await?;
        return Ok(());
    }

    if args.serve {
        background::start_library_watcher(
            state.library.clone(),
            state.device_service(),
            state.settings.clone(),
            state.download_manager.clone(),
            state.events.clone(),
        )
        .await?;

        state.device_service().init_tracker();

        api::run_server(state, &args).await
    } else if args.sync_player {
        state.device_service().sync_configured_players().await?;
        Ok(())
    } else if let Some(ref url) = args.url {
        cli::run_download(&state, url, &args.proxies).await
    } else {
        Ok(())
    }
}

pub async fn init_app(args: &Args) -> Result<AppState> {
    infra::db::init()?;
    let mut settings = infra::db::SettingsRepo::get()?;
    let output_dir = args.resolve_output_dir(&settings);
    settings.downloads.output_path = output_dir.to_string_lossy().to_string();
    fs::create_dir_all(&output_dir)?;

    let pb = terminal::create_standalone_spinner("Initializing SoundCloud...");
    let library = Arc::new(RwLock::new(Library::default()));
    let settings_manager = SettingsManager::new(settings.clone());
    let library_service = LibraryService::new(library, settings_manager.clone());
    let client = ClientBuilder::new(&settings)
        .with_settings_manager(settings_manager.clone())
        .build()
        .await?;
    pb.finish_with_message("SoundCloud ready");

    let state = AppState::new(client, library_service, settings_manager);

    let pb_idx = terminal::create_standalone_spinner("Indexing local library...");
    state.library_service().reindex().await;
    pb_idx.finish_with_message("Local library indexed");

    Ok(state)
}

pub mod errors;
pub mod handlers;
pub mod state;
pub mod static_files;
pub mod types;

use std::net::SocketAddr;

use axum::{
    Router,
    routing::{delete, get, post},
    serve,
};
use handlers::{
    devices::{
        devices_ws, get_device_storage_info, list_adb_devices, scan_adb_device, sync_adb_device,
    },
    download::{
        download_events, get_download_queue, get_syncing_urls, remove_from_queue, start_download,
    },
    search::{
        search_tracks,
        types::{SearchResponse, SearchTrackItem},
    },
    settings::{
        config::{get_settings, update_settings},
        diagnostics::{ProxyTestResult, TestSoundCloudRequest, test_proxy_ws, test_soundcloud},
        sync::{SnapshotResponse, SyncRequest, get_snapshot_handler, sync_handler},
    },
    stream::stream_audio,
    tracks::{get_tracks, reindex_library, remove_track, remove_tracks},
};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use types::{ApiStatus, DownloadRequest, DownloadStartResponse, MusicTrackRecord};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    adb::{StorageInfo, StorageType},
    api::state::AppState,
    backup::{BackupAction, BackupProvider, WebDavBackupConfig},
    cli::Args,
    download_manager::{DownloadItem, DownloadStatus, MessageLevel},
    settings::{
        AdbDeviceSettings, AdbSettings, DownloadSettings, NetworkSettings, SoundcloudSettings,
        SystemSettings, UserSettings, WebDavSettings,
    },
    types::AudioFormat,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::download::queue::start_download,
        handlers::download::queue::get_download_queue,
        handlers::download::queue::remove_from_queue,
        handlers::download::events::download_events,
        handlers::download::queue::get_syncing_urls,
        handlers::stream::stream_audio,
        handlers::search::tracks::search_tracks,
        handlers::tracks::list::get_tracks,
        handlers::tracks::library::reindex_library,
        handlers::tracks::manage::remove_track,
        handlers::tracks::manage::remove_tracks,
        handlers::devices::list::list_adb_devices,
        handlers::devices::ws::devices_ws,
        handlers::devices::list::get_device_storage_info,
        handlers::devices::sync::sync_adb_device,
        handlers::devices::scan::scan_adb_device,
        handlers::settings::config::get_settings,
        handlers::settings::config::update_settings,
        handlers::settings::sync::sync_handler,
        handlers::settings::sync::get_snapshot_handler,
        handlers::settings::diagnostics::test_soundcloud,
        handlers::settings::diagnostics::test_proxy_ws,
    ),
    components(
        schemas(
            // API DTOs
            ApiStatus,
            AudioFormat,
            DownloadRequest,
            DownloadStartResponse,
            MusicTrackRecord,

            // Settings
            UserSettings,
            SoundcloudSettings,
            DownloadSettings,
            AdbSettings,
            AdbDeviceSettings,
            NetworkSettings,
            SystemSettings,
            WebDavSettings,

            // Storage & ADB
            StorageInfo,
            StorageType,

            // Download Manager
            DownloadItem,
            DownloadStatus,
            MessageLevel,

            // Diagnostics & Sync
            ProxyTestResult,
            SnapshotResponse,
            SyncRequest,
            TestSoundCloudRequest,

            // Backup
            BackupAction,
            BackupProvider,
            WebDavBackupConfig,

            // Search & Tracks
            SearchResponse,
            SearchTrackItem,
        )
    ),
    tags(
        (name = "VortexDL", description = "VortexDL REST API")
    )
)]
struct ApiDoc;

pub async fn run_server(state: AppState, args: &Args) -> anyhow::Result<()> {
    let router = build_router(state, args.serve);
    let addr = SocketAddr::new(args.host, args.port);
    let listener = TcpListener::bind(addr).await?;

    tracing::info!("VortexDL running on http://{}", addr);
    serve(listener, router).await?;
    Ok(())
}

pub fn build_router(state: AppState, embed_frontend: bool) -> Router {
    let api_routes = Router::new()
        .route("/stream/{id}", get(stream_audio))
        .nest("/download", download_routes())
        .nest("/downloads", downloads_routes())
        .nest("/library", library_routes())
        .nest("/search", search_routes())
        .nest("/devices", devices_routes())
        .nest("/settings", settings_routes());

    let router = Router::new()
        .nest("/api", api_routes)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state);

    let router = if embed_frontend {
        router.fallback(static_files::static_handler)
    } else {
        router
    };

    router.layer(CorsLayer::permissive())
}

fn download_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(start_download))
        .route("/queue", get(get_download_queue))
        .route("/queue/{id}", delete(remove_from_queue))
        .route("/events", get(download_events))
        .route("/syncing", get(get_syncing_urls))
}

fn downloads_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_tracks).delete(remove_tracks))
        .route("/{id}", delete(remove_track))
}

fn library_routes() -> Router<AppState> {
    Router::new().route("/reindex", post(reindex_library))
}

fn search_routes() -> Router<AppState> {
    Router::new().route("/tracks", get(search_tracks))
}

fn devices_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_adb_devices))
        .route("/ws", get(devices_ws))
        .route("/{device_id}/storage", get(get_device_storage_info))
        .route("/{device_id}/sync", post(sync_adb_device))
        .route("/{device_id}/scan", post(scan_adb_device))
}

fn settings_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_settings).post(update_settings))
        .route("/sync", post(sync_handler))
        .route("/sync/snapshot", get(get_snapshot_handler))
        .route("/test/soundcloud", post(test_soundcloud))
        .route("/test/proxy/ws", get(test_proxy_ws))
}

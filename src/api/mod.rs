use std::net::SocketAddr;

use axum::{
    Router,
    routing::{delete, get, post},
    serve,
};
use handlers::{
    devices::{devices_ws, get_device_storage_info, list_adb_devices, sync_adb_device},
    download::{
        download_events, get_download_queue, get_syncing_urls, remove_from_queue, start_download,
    },
    search::{get_stream_url, search_tracks},
    settings::{
        config::{get_settings, update_settings},
        diagnostics::{test_proxy_ws, test_soundcloud},
        sync::{get_snapshot_handler, sync_handler},
    },
    tracks::{get_tracks, reindex_library, remove_track, remove_tracks, stream_track},
};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{api::state::AppState, cli::Args};

pub mod download_manager;
pub mod errors;
pub mod handlers;
pub mod state;
pub mod static_files;
pub mod types;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::download::start_download,
        handlers::download::get_download_queue,
        handlers::download::remove_from_queue,
        handlers::download::download_events,
        handlers::download::get_syncing_urls,
        handlers::search::search_tracks,
        handlers::search::get_stream_url,
        handlers::tracks::get_tracks,
        handlers::tracks::reindex_library,
        handlers::tracks::remove_track,
        handlers::tracks::remove_tracks,
        handlers::tracks::stream_track,
        handlers::devices::list_adb_devices,
        handlers::devices::devices_ws,
        handlers::devices::get_device_storage_info,
        handlers::devices::sync_adb_device,
        handlers::settings::config::get_settings,
        handlers::settings::config::update_settings,
        handlers::settings::sync::sync_handler,
        handlers::settings::sync::get_snapshot_handler,
        handlers::settings::diagnostics::test_soundcloud,
        handlers::settings::diagnostics::test_proxy_ws,
    ),
    components(
        schemas(
            types::DownloadRequest,
            types::ApiStatus,
            types::DownloadStartResponse,
            types::AudioFormat,
            types::MusicTrackRecord,
            crate::settings::SoundcloudSettings,
            crate::settings::DownloadSettings,
            crate::settings::AdbDeviceSettings,
            crate::settings::AdbSettings,
            crate::settings::NetworkSettings,
            crate::settings::SystemSettings,
            crate::settings::UserSettings,
            crate::adb::StorageType,
            crate::adb::StorageInfo,
            download_manager::DownloadStatus,
            download_manager::DownloadTrackDetails,
            download_manager::DownloadItem,
            download_manager::MessageLevel,
            handlers::settings::diagnostics::TestSoundCloudRequest,
            handlers::settings::diagnostics::ProxyTestResult,
            handlers::settings::sync::SyncRequest,
            handlers::settings::sync::SyncAction,
            handlers::settings::sync::SyncProvider,
            handlers::settings::sync::SnapshotResponse,
            handlers::search::SearchTrackItem,
            handlers::search::SearchResponse,
            handlers::search::StreamUrlResponse,
            handlers::tracks::DeleteTracksPayload,
        )
    ),
    tags(
        (name = "VortexDL", description = "VortexDL REST API")
    )
)]
struct ApiDoc;

pub async fn run_server(state: AppState, args: &Args) -> anyhow::Result<()> {
    let router = build_router(state, args.serve).await;
    let addr = SocketAddr::new(args.host, args.port);
    let listener = TcpListener::bind(addr).await?;

    tracing::info!("VortexDL running on http://{}", addr);
    serve(listener, router).await?;
    Ok(())
}

pub async fn build_router(state: AppState, embed_frontend: bool) -> Router {
    let api_routes = Router::new()
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
        .route("/{id}/stream", get(stream_track))
}

fn library_routes() -> Router<AppState> {
    Router::new().route("/reindex", post(reindex_library))
}

fn search_routes() -> Router<AppState> {
    Router::new()
        .route("/tracks", get(search_tracks))
        .route("/tracks/{id}/stream", get(get_stream_url))
}

fn devices_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_adb_devices))
        .route("/ws", get(devices_ws))
        .route("/{device_id}/storage", get(get_device_storage_info))
        .route("/{device_id}/sync", post(sync_adb_device))
}

fn settings_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_settings).post(update_settings))
        .route("/sync", post(sync_handler))
        .route("/sync/snapshot", get(get_snapshot_handler))
        .route("/test/soundcloud", post(test_soundcloud))
        .route("/test/proxy/ws", get(test_proxy_ws))
}

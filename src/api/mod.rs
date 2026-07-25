use std::net::SocketAddr;

use axum::{
    Router,
    routing::{delete, get, post},
    serve,
};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    api::{
        handlers::{
            devices::{devices_ws, get_device_storage_info, list_adb_devices, sync_adb_device},
            download::{
                download_events, get_download_queue, get_syncing_urls, remove_from_queue,
                start_download,
            },
            health::health,
            search::{get_stream_url, search_tracks},
            settings::{
                diagnostics::{test_proxy_ws, test_soundcloud},
                get_settings, update_settings,
            },
            tracks::{get_tracks, reindex_library, remove_track, remove_tracks, stream_track},
        },
        state::AppState,
    },
    cli::Args,
};

pub mod download_manager;
pub mod errors;
pub mod handlers;
pub mod state;
pub mod static_files;
pub mod types;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::handlers::health::health,
        crate::api::handlers::download::start_download,
        crate::api::handlers::download::get_download_queue,
        crate::api::handlers::download::remove_from_queue,
        crate::api::handlers::download::download_events,
        crate::api::handlers::download::get_syncing_urls,
        crate::api::handlers::search::search_tracks,
        crate::api::handlers::search::get_stream_url,
        crate::api::handlers::tracks::get_tracks,
        crate::api::handlers::tracks::reindex_library,
        crate::api::handlers::tracks::remove_track,
        crate::api::handlers::tracks::remove_tracks,
        crate::api::handlers::tracks::stream_track,
        crate::api::handlers::devices::list_adb_devices,
        crate::api::handlers::devices::devices_ws,
        crate::api::handlers::devices::get_device_storage_info,
        crate::api::handlers::devices::sync_adb_device,
        crate::api::handlers::settings::get_settings,
        crate::api::handlers::settings::update_settings,
        crate::api::handlers::settings::diagnostics::test_soundcloud,
    ),
    components(
        schemas(
            crate::api::types::DownloadRequest,
            crate::api::types::ApiStatus,
            crate::api::types::DownloadStartResponse,
            crate::api::types::AudioFormat,
            crate::api::types::MusicTrackRecord,
            crate::api::types::HealthResponse,
            crate::settings::SoundcloudSettings,
            crate::settings::DownloadSettings,
            crate::settings::AdbDeviceSettings,
            crate::settings::AdbSettings,
            crate::settings::NetworkSettings,
            crate::settings::UserSettings,
            crate::adb_device::StorageType,
            crate::adb_device::StorageInfo,
            crate::api::download_manager::DownloadStatus,
            crate::api::download_manager::DownloadItem,
            crate::api::handlers::settings::diagnostics::TestSoundCloudRequest,
            crate::api::handlers::settings::diagnostics::ProxyTestResult,
            crate::api::handlers::search::SearchTrackItem,
            crate::api::handlers::search::SearchResponse,
            crate::api::handlers::search::StreamUrlResponse,
            crate::api::handlers::tracks::DeleteTracksPayload,
        )
    ),
    tags(
        (name = "VortexDL", description = "VortexDL REST API")
    )
)]
struct ApiDoc;

pub async fn run_server(state: AppState, args: &Args) -> anyhow::Result<()> {
    let router = build_router(state, args.serve).await;
    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;
    let listener = TcpListener::bind(addr).await?;

    tracing::info!("VortexDL running on http://{}", addr);
    serve(listener, router).await?;
    Ok(())
}

pub async fn build_router(state: AppState, embed_frontend: bool) -> Router {
    let api_routes = Router::new()
        .route("/health", get(health))
        .nest("/download", download_routes())
        .route("/downloads", get(get_tracks).delete(remove_tracks))
        .route("/downloads/{id}", delete(remove_track))
        .route("/downloads/{id}/stream", get(stream_track))
        .route("/library/reindex", post(reindex_library))
        .route("/search/tracks", get(search_tracks))
        .route("/search/tracks/{id}/stream", get(get_stream_url))
        .route("/devices", get(list_adb_devices))
        .route("/devices/ws", get(devices_ws))
        .route("/devices/{device_id}/storage", get(get_device_storage_info))
        .route("/devices/{device_id}/sync", post(sync_adb_device))
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

fn settings_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_settings).post(update_settings))
        .route("/test/soundcloud", post(test_soundcloud))
        .route("/test/proxy/ws", get(test_proxy_ws))
}

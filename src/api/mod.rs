use std::net::SocketAddr;

use axum::{
    Router,
    routing::{delete, get, post},
    serve,
};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

use crate::{
    api::{
        handlers::{
            devices::list_adb_devices,
            download::{download_events, get_download_queue, remove_from_queue, start_download},
            health::health,
            settings::{get_settings, test::test_soundcloud_url, update_settings},
            tracks::{get_tracks, indexing_tracks, remove_track, stream_track},
        },
        state::AppState,
    },
    cli::Args,
};

pub mod download_manager;
pub mod errors;
pub mod handlers;
pub mod models;
pub mod state;
#[cfg(feature = "web")]
pub mod static_files;

pub async fn run_server(state: AppState, args: &Args) -> anyhow::Result<()> {
    let router = build_router(state, args.serve).await;
    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;
    let listener = TcpListener::bind(addr).await?;

    println!("VortexDL running on http://{}", addr);
    serve(listener, router).await?;
    Ok(())
}

pub async fn build_router(state: AppState, serve_frontend: bool) -> Router {
    let api_routes = Router::new()
        .route("/health", get(health))
        .nest("/download", download_routes())
        .route("/downloads", get(get_tracks))
        .route("/downloads/{id}", delete(remove_track))
        .route("/downloads/{id}/stream", get(stream_track))
        .route("/downloads/indexing_tracks", get(indexing_tracks))
        .route("/devices", get(list_adb_devices))
        .nest("/settings", settings_routes());

    let router = Router::new().nest("/api", api_routes).with_state(state);

    if serve_frontend {
        #[cfg(feature = "web")]
        {
            router = router.fallback(static_files::static_handler);
        }
        #[cfg(not(feature = "web"))]
        {
            tracing::warn!("Frontend requested but binary built without 'web' feature");
        }
    }

    router.layer(CorsLayer::permissive())
}

fn download_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(start_download))
        .route("/queue", get(get_download_queue))
        .route(
            "/queue/{id}",
            get(remove_from_queue).delete(remove_from_queue),
        )
        .route("/events", get(download_events))
}

fn settings_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_settings).post(update_settings))
        .route("/test/soundcloud", post(test_soundcloud_url))
}

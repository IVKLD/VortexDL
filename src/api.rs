use axum::{
    Router,
    routing::{delete, get, post},
};
use tower_http::cors::CorsLayer;

use crate::api::{
    handlers::{
        download::{download_events, get_download_queue, remove_from_queue, start_download},
        health::health,
        settings::{get_settings, test::test_soundcloud_url, update_settings},
        tracks::{get_tracks, indexing_tracks, remove_track},
    },
    state::AppState,
};

pub mod download_manager;
pub mod errors;
pub mod handlers;
pub mod models;
pub mod state;
pub mod static_files;

pub async fn build_router(state: AppState, serve_frontend: bool) -> Router {
    let api_routes = Router::new()
        .route("/health", get(health))
        .nest("/download", download_routes())
        .route("/downloads", get(get_tracks))
        .route("/downloads/{id}", delete(remove_track))
        .route("/downloads/indexing_tracks", get(indexing_tracks))
        .nest("/settings", settings_routes());

    let mut router = Router::new().nest("/api", api_routes).with_state(state);

    if serve_frontend {
        router = router.fallback(static_files::static_handler);
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

pub mod errors;
pub mod models;
pub mod routes;
pub mod state;

use axum::{
    Router,
    routing::{delete, get, post},
};
use tower_http::cors::CorsLayer;

use routes::{delete_download, download_likes, download_playlist, health, list_downloads};

pub fn build_router(state: state::AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/download/playlist", post(download_playlist))
        .route("/download/likes", post(download_likes))
        .route("/downloads", get(list_downloads))
        .route("/downloads/:filename", delete(delete_download))
        .with_state(state)
        .layer(CorsLayer::permissive())
}

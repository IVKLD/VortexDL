pub mod errors;
pub mod models;
pub mod state;
pub mod download_manager;
pub mod api;
pub mod static_files;

use axum::{
    Router,
    routing::{get, post},
};
use tower_http::cors::CorsLayer;

use api::{
    download::{download_events, get_download_queue, remove_from_queue, start_download},
    health::health,
    tracks::{get_tracks, remove_track},
};

pub async fn build_router(state: state::AppState, serve_frontend: bool) -> Router {
    let api_routes = Router::new()
        .route("/health", get(health))
        .route("/download", post(start_download))
        .route("/download/queue", get(get_download_queue))
        .route("/download/queue/{id}", get(remove_from_queue).delete(remove_from_queue))
        .route("/download/events", get(download_events))
        .route("/downloads", get(get_tracks).delete(remove_track));

    let mut router = Router::new()
        .nest("/api", api_routes)
        .with_state(state);

    if serve_frontend {
        router = router.fallback(static_files::static_handler);
    }

    router.layer(CorsLayer::permissive())
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::state::AppState;
    use crate::config::AppConfig;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use soundcloud_rs::ClientBuilder;
    use tempfile::tempdir;
    use tower::util::ServiceExt;

    async fn setup() -> Router {
        let config = AppConfig::default();
        let client = ClientBuilder::new().build().await.unwrap();
        let dir = tempdir().unwrap();
        let output_dir = dir.path().to_str().unwrap().to_string();
        let state = AppState::new(client, config, output_dir);
        build_router(state, false).await
    }

    #[tokio::test]
    async fn test_health_check() {
        let router = setup().await;
        let response = router
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}

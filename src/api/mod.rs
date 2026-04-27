pub mod errors;
pub mod handlers;
pub mod models;
pub mod state;
pub mod download_manager;

use axum::{
    Router,
    routing::{get, post},
};
use tower_http::cors::CorsLayer;

use handlers::{
    download::{download_events, get_download_queue, process_download, queue_likes, queue_playlist, queue_track},
    health::health,
    tracks::{get_tracks, remove_track},
};

pub async fn build_router(state: state::AppState) -> Router {
    {
        let mut storage = state.storage.write().await;
        storage.indexing(std::path::Path::new(state.output_dir.as_str()));
    }

    Router::new()
        .route("/health", get(health))
        .route("/download", post(process_download))
        .route("/download/track", post(queue_track))
        .route("/download/playlist", post(queue_playlist))
        .route("/download/likes", post(queue_likes))
        .route("/download/queue", get(get_download_queue))
        .route("/download/events", get(download_events))
        .route("/downloads", get(get_tracks).delete(remove_track))
        .with_state(state)
        .layer(CorsLayer::permissive())
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
        build_router(state).await
    }

    #[tokio::test]
    async fn test_health_check() {
        let router = setup().await;
        let response = router
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}

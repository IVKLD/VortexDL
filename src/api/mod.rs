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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{Request, StatusCode};
    use axum::body::Body;
    use tower::ServiceExt;
    use crate::config::AppConfig;
    use crate::api::state::AppState;
    use soundcloud_rs::ClientBuilder;
    use std::sync::Arc;
    use tempfile::tempdir;

    async fn setup() -> Router {
        let config = AppConfig::default();
        let client = ClientBuilder::new().build().await.unwrap();
        let dir = tempdir().unwrap();
        let output_dir = dir.path().to_str().unwrap().to_string();
        let state = AppState::new(client, config, output_dir);
        build_router(state)
    }

    #[tokio::test]
    async fn test_health_check() {
        let router = setup().await;
        let response = router
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}

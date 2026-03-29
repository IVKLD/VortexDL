use std::sync::Arc;

use soundcloud_rs::Client;

use crate::config::AppConfig;

/// Shared application state injected into every Axum handler.
#[derive(Clone)]
pub struct AppState {
    /// Authenticated SoundCloud API client.
    pub client: Arc<Client>,
    /// Application configuration.
    pub config: Arc<AppConfig>,
    /// Directory where downloaded tracks are stored.
    pub output_dir: Arc<String>,
}

impl AppState {
    pub fn new(client: Client, config: AppConfig, output_dir: String) -> Self {
        Self {
            client: Arc::new(client),
            config: Arc::new(config),
            output_dir: Arc::new(output_dir),
        }
    }
}

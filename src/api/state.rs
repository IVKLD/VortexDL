use std::sync::Arc;

use soundcloud_rs::Client;

use crate::config::AppConfig;


#[derive(Clone)]
pub struct AppState {
    
    pub client: Arc<Client>,
    
    pub config: Arc<AppConfig>,
    
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

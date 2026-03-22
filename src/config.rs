use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub client_id: Option<String>,
    pub oauth_token: Option<String>,
    pub user_id: Option<i64>,
    pub locale: String,
    pub user_agent: String,
    pub default_output_dir: String,
    pub limit_per_page: u32,
    pub max_retries: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            client_id: None,
            oauth_token: None,
            user_id: None,
            locale: String::from("en_US"),
            user_agent: String::from("Mozilla/5.0 (Windows NT 10.0; Win64; x64)"),
            default_output_dir: String::from("./downloads"),
            limit_per_page: 50,
            max_retries: 5,
        }
    }
}

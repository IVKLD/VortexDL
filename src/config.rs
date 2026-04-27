use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
            limit_per_page: 100,
            max_retries: 5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.locale, "en_US");
        assert_eq!(config.limit_per_page, 100);
        assert_eq!(config.default_output_dir, "./downloads");
    }

    #[test]
    fn test_serialization() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.locale, config.locale);
    }
}

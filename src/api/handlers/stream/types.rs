use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct StreamQuery {
    pub url: Option<String>,
}

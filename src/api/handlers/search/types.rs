use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum SearchProviderParam {
    Youtube,
    Soundcloud,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum SearchDurationFilter {
    Any,
    Short,
    Medium,
    Long,
    Epic,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub provider: Option<SearchProviderParam>,
    pub duration: Option<SearchDurationFilter>,
}

#[derive(Debug, Serialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SearchTrackItem {
    pub id: i64,
    pub title: String,
    pub artist: String,
    pub artwork_url: Option<String>,
    pub duration: Option<i64>,
    pub playback_count: Option<i64>,
    pub permalink_url: Option<String>,
    pub genre: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    pub tracks: Vec<SearchTrackItem>,
    pub has_more: bool,
}

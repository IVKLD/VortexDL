use serde::{Deserialize, Serialize};

pub const SC_IDENTIFIER: &str = "sc-identifier";
pub const SC_ARTWORK_URL: &str = "sc-artwork-url";

#[derive(Serialize)]
pub struct ResolveQuery {
    pub url: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ResolveResponse {
    pub id: i64,
    pub kind: String,
    pub title: Option<String>,
    pub artwork_url: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct TrackLikesResponse {
    pub collection: Vec<LikeItem>,
    pub next_href: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct LikeItem {
    pub track: TrackInfo,
}

#[derive(Deserialize, Debug)]
pub struct TrackInfo {
    pub id: i64,
    pub title: String,
    pub artwork_url: Option<String>,
}

#[derive(Serialize)]
pub struct TrackLikesQuery {
    pub limit: u32,
    pub offset: Option<String>,
}

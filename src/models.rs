use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct ResolveQuery {
    pub url: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ResolveResponse {
    pub id: i64,
    pub kind: String,
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
}

#[derive(Serialize)]
pub struct TrackLikesQuery {
    pub limit: u32,
    // pub linked_partitioning: u8,
    pub offset: Option<String>,
}

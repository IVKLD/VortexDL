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
    pub artwork_url: Option<String>,
    pub user: Option<UserInfo>,
}

#[derive(Deserialize, Debug)]
pub struct UserInfo {
    pub username: String,
}

#[derive(Serialize)]
pub struct TrackLikesQuery {
    pub limit: u32,
    pub offset: Option<String>,
}

pub trait AsUsername {
    fn username(&self) -> Option<&str>;
}

impl AsUsername for UserInfo {
    fn username(&self) -> Option<&str> {
        Some(&self.username)
    }
}

impl AsUsername for soundcloud_rs::UserSummary {
    fn username(&self) -> Option<&str> {
        self.username.as_deref()
    }
}


use std::fmt;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(ValueEnum, Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
#[value(rename_all = "lowercase")]
pub enum SyncMode {
    Silent,
    Full,
    Archive,
}

impl fmt::Display for SyncMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Silent => write!(f, "silent"),
            Self::Full => write!(f, "full"),
            Self::Archive => write!(f, "archive"),
        }
    }
}

#[derive(Serialize)]
pub struct ResolveQuery {
    pub url: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ResolveResponse {
    pub id: i64,
    pub kind: String,
    #[allow(dead_code)]
    pub title: Option<String>,
    #[allow(dead_code)]
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

use serde::{Deserialize, Serialize};



#[derive(Debug, Deserialize)]
pub struct DownloadPlaylistRequest {
    
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct DownloadLikesRequest {
    
    pub url: String,
}



#[derive(Debug, Serialize)]
pub struct DownloadQueuedResponse {
    pub status: &'static str,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct DownloadedTrack {
    pub filename: String,
}

#[derive(Debug, Serialize)]
pub struct ListResponse {
    pub tracks: Vec<DownloadedTrack>,
}

#[derive(Debug, Serialize)]
pub struct DeleteResponse {
    pub deleted: bool,
    pub filename: String,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
}

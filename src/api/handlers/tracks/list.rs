use axum::{
    Json,
    extract::{Query, State},
    response::IntoResponse,
};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::api::{errors::ApiError, state::AppState, types::MusicTrackRecord};

#[derive(Deserialize, IntoParams)]
pub struct TracksQuery {
    pub sort: Option<String>,
    pub order: Option<String>,
    pub limit: Option<usize>,
}

#[utoipa::path(
    method(get),
    path = "/api/downloads",
    params(
        TracksQuery
    ),
    responses(
        (status = 200, description = "Get list of local track records", body = Vec<MusicTrackRecord>)
    )
)]
pub async fn get_tracks(
    State(state): State<AppState>,
    Query(query): Query<TracksQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let storage = state.storage.read().await;

    let mut tracks = storage
        .tracks
        .iter()
        .map(|(id, data)| MusicTrackRecord::from_local_track(*id, data))
        .collect::<Vec<_>>();

    let sort = query.sort.as_deref().unwrap_or("date");
    let order = query.order.as_deref().unwrap_or("desc");

    tracks.sort_by(|a, b| {
        let cmp = match sort {
            "name" => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
            _ => b.created_at.cmp(&a.created_at),
        };

        let cmp = cmp.then_with(|| a.id.cmp(&b.id));

        if order == "desc" { cmp.reverse() } else { cmp }
    });

    if let Some(limit) = query.limit {
        tracks.truncate(limit);
    }

    Ok(Json(tracks))
}

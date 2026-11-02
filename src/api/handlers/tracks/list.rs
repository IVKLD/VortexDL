use axum::{
    Json,
    extract::{Query, State},
    response::IntoResponse,
};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::{
    api::{errors::ApiError, types::TrackRecord},
    app::AppState,
};

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
        (status = 200, description = "Get list of local track records", body = Vec<TrackRecord>)
    )
)]
pub async fn get_tracks(
    State(state): State<AppState>,
    Query(query): Query<TracksQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let tracks = state
        .library_service()
        .list_tracks(query.sort.as_deref(), query.order.as_deref(), query.limit)
        .await;

    let records: Vec<TrackRecord> = tracks
        .into_iter()
        .map(|(id, data)| TrackRecord::from_track(id, &data))
        .collect();

    Ok(Json(records))
}

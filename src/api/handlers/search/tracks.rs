use axum::{
    Json,
    extract::{Query, State},
    response::IntoResponse,
};

use crate::{
    api::errors::ApiError,
    app::AppState,
    search::{SearchDurationFilter, SearchProviderParam, SearchQuery, SearchResponse},
};

#[utoipa::path(
    method(get),
    path = "/api/search/tracks",
    params(
        ("query" = String, Query, description = "Search query"),
        ("limit" = Option<i32>, Query, description = "Max results to return"),
        ("offset" = Option<i32>, Query, description = "Result offset for pagination"),
        ("provider" = Option<SearchProviderParam>, Query, description = "Search provider filter: youtube, soundcloud"),
        ("duration" = Option<SearchDurationFilter>, Query, description = "Duration filter: any, short, medium, long, epic")
    ),
    responses(
        (status = 200, description = "Search results", body = SearchResponse)
    )
)]
pub async fn search_tracks(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let response = state.search_service().search(params).await?;
    Ok(Json(response))
}

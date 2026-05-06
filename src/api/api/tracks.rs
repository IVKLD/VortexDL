use axum::{Json, extract::{Query, State}, response::IntoResponse};
use std::fs;
use crate::api::{
    errors::ApiError,
    models::{KNOWN_EXTENSIONS, DeleteQuery, TrackExtension, TrackRecord},
    state::AppState,
};

pub async fn get_tracks(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    {
        let mut storage = state.storage.write().await;
        storage.indexing(std::path::Path::new(state.output_dir.as_str()));
    }
    
    let storage = state.storage.read().await;
    
    let tracks = storage.tracks.iter()
        .filter_map(|(id, data)| {
            let path = &data.path;
            let extension = path
                .extension()
                .map(|ext| ext.to_string_lossy())
                .map(|ext| match ext.as_ref() {
                    "mp3" => TrackExtension::MP3,
                    "flac" => TrackExtension::FLAC,
                    "wav" => TrackExtension::WAV,
                    _ => TrackExtension::Unknown,
                })
                .unwrap_or(TrackExtension::Unknown);

            if path.is_file() && KNOWN_EXTENSIONS.contains(&extension) {
                let file_name = path.file_name()?;

                Some(TrackRecord {
                    id: *id as u32,
                    filename: file_name.to_string_lossy().into_owned(),
                    album: String::new(),
                    format: path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or_default()
                        .to_string(),
                    artwork_url: data.artwork_url.clone(),
                    source_url: data.source_url.clone(),
                    created_at: path.metadata()
                        .and_then(|m| m.created())
                        .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())
                        .unwrap_or(0),
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    Ok(Json(tracks))
}

pub async fn remove_track(
    State(state): State<AppState>,
    Query(body): Query<DeleteQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let target_id = body.id as i64;
    
    let path = {
        let storage = state.storage.read().await;
        storage.tracks.get(&target_id).cloned()
    };

    if let Some(data) = path {
        if data.path.exists() {
            fs::remove_file(&data.path)
                .map_err(|e| ApiError::internal(format!("Failed to delete file: {e}")))?;
        }
        
        let mut storage = state.storage.write().await;
        storage.remove_track(target_id);
        
        return Ok(());
    }

    Err(ApiError::not_found(format!("Track with ID {} not found", body.id)))
}

use crate::{
    api::errors::{ApiError, ErrorCode},
    providers::{SoundCloudProvider, YouTubeProvider},
    search::{
        cache::SearchCache,
        types::{
            SearchDurationFilter, SearchProviderParam, SearchQuery, SearchResponse, SearchTrackItem,
        },
    },
};

#[derive(Clone)]
pub struct SearchService {
    soundcloud: SoundCloudProvider,
    youtube: YouTubeProvider,
    cache: SearchCache,
}

impl SearchService {
    pub fn new(
        soundcloud: SoundCloudProvider,
        youtube: YouTubeProvider,
        cache: SearchCache,
    ) -> Self {
        Self {
            soundcloud,
            youtube,
            cache,
        }
    }

    pub async fn search(&self, params: SearchQuery) -> Result<SearchResponse, ApiError> {
        let trimmed = params.query.trim();
        let query = (!trimmed.is_empty()).then_some(trimmed).ok_or_else(|| {
            ApiError::bad_request("Search query cannot be empty").with_code(ErrorCode::BadRequest)
        })?;

        let limit = (params.limit.unwrap_or(20).max(1) as usize).min(100);
        let offset = params.offset.unwrap_or(0).max(0) as usize;
        let provider = params.provider.unwrap_or(SearchProviderParam::Soundcloud);
        let duration = params.duration.unwrap_or(SearchDurationFilter::Any);

        match provider {
            SearchProviderParam::Youtube => {
                self.search_youtube(query, limit, offset, duration).await
            }
            SearchProviderParam::Soundcloud => self.search_soundcloud(query, limit, offset).await,
        }
    }

    async fn search_youtube(
        &self,
        query: &str,
        limit: usize,
        offset: usize,
        duration: SearchDurationFilter,
    ) -> Result<SearchResponse, ApiError> {
        if offset == 0
            && self.youtube.is_youtube_url(query)
            && let Some(meta) = self.youtube.fetch_metadata(query).await
        {
            let (id, vid, item) = SearchTrackItem::from_youtube_meta(&meta);
            self.cache.insert_youtube_id(id, vid);
            return Ok(SearchResponse {
                tracks: vec![item],
                has_more: false,
            });
        }

        let continuation = if offset > 0 {
            self.cache.continuation(query)
        } else {
            None
        };

        let Some((videos, next_token)) = self
            .youtube
            .search_page(query, continuation.as_deref())
            .await
        else {
            return Ok(SearchResponse::empty());
        };

        let has_more = next_token.is_some();
        self.cache.set_continuation(query, next_token);

        let mut tracks = Vec::new();
        for meta in videos {
            if !duration.matches(meta.duration_seconds) {
                continue;
            }

            let (id, vid, item) = SearchTrackItem::from_youtube_meta(&meta);
            self.cache.insert_youtube_id(id, vid);
            tracks.push(item);
            if tracks.len() >= limit {
                break;
            }
        }

        Ok(SearchResponse { tracks, has_more })
    }

    async fn search_soundcloud(
        &self,
        query: &str,
        limit: usize,
        offset: usize,
    ) -> Result<SearchResponse, ApiError> {
        let Some(sc_results) = self.soundcloud.search_tracks(query, limit, offset).await else {
            return Ok(SearchResponse::empty());
        };

        let has_more = sc_results.next_href.is_some();
        let mut tracks = Vec::new();

        for track in sc_results.collection {
            let Some(id) = track.id else { continue };
            self.cache.insert_soundcloud_track(id, track.clone());
            tracks.push(SearchTrackItem::from_soundcloud_track(id, track));
        }

        Ok(SearchResponse { tracks, has_more })
    }
}

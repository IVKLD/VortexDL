use soundcloud_rs::{Tracks, TracksQuery};

use crate::providers::soundcloud::SoundCloudProvider;

impl SoundCloudProvider {
    pub async fn search_tracks(
        &self,
        query: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Tracks, soundcloud_rs::Error> {
        let sc_query = TracksQuery {
            q: Some(query.to_string()),
            limit: Some(limit as i32),
            offset: Some(offset as i32),
            ..Default::default()
        };

        self.client.search_tracks(Some(&sc_query)).await
    }
}

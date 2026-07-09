use std::collections::HashSet;
use url::Url;
use anyhow::Result;
use crate::database::get_previous_ids;
use super::MusicStorage;

impl MusicStorage {
    pub async fn sync_storage(
        &mut self,
        url: &Url,
        current_soundcloud_ids: &HashSet<i64>,
    ) -> Result<()> {
        let url_str = url.as_str();
        let prev_ids = get_previous_ids(url_str)?;
        let accumulated_ids: HashSet<i64> = prev_ids.union(current_soundcloud_ids).copied().collect();
        crate::database::save_sync_ids(url_str, &accumulated_ids)?;
        Ok(())
    }
}
